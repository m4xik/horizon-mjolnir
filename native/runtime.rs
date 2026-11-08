// Mjolnir runtime: pure logic, no game types, so it also builds as a standalone library.
// Hooks queue requests here; the server tick drains the due actions and the helper applies them.
use std::collections::HashMap;
use std::sync::Mutex;

pub const ITEM: &str = "mjolnir:mjolnir";
const WRATH_COOLDOWN_TICKS: u64 = 100;

#[derive(Clone, Debug, PartialEq)]
pub enum Action {
    // A vanilla command run as the server console.
    Command(String),
    // A lightning bolt summoned in `dim` at `pos`.
    Bolt { dim: String, pos: [f64; 3], big: bool },
    // A jagged lightning arc drawn on the client from `from` to `to`.
    Arc { from: [f64; 3], to: [f64; 3] },
}

#[derive(Default)]
struct State {
    tick: u64,
    pending: Vec<(u64, Action, Instant)>,
    last_wrath: HashMap<u128, u64>,
}

static STATE: Mutex<Option<State>> = Mutex::new(None);

fn with<R>(f: impl FnOnce(&mut State) -> R) -> R {
    let mut guard = STATE.lock().unwrap_or_else(|e| e.into_inner());
    f(guard.get_or_insert_with(State::default))
}

// Only plain player names may reach a command line.
fn safe_name(name: &str) -> bool {
    !name.is_empty() && name.len() <= 16 && name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
}

fn at(dim: &str, body: String) -> Action {
    Action::Command(format!("execute in {dim} run {body}"))
}

fn sparks(dim: &str, p: [f64; 3], spread: f64, count: u32) -> Action {
    at(
        dim,
        format!(
            "particle minecraft:electric_spark {:.2} {:.2} {:.2} {spread} {spread} {spread} 0.9 {count} force",
            p[0],
            p[1] + 1.0,
            p[2]
        ),
    )
}

fn sound(dim: &str, p: [f64; 3], name: &str, volume: f32, pitch: f32) -> Action {
    at(
        dim,
        format!(
            "playsound {name} master @a {:.2} {:.2} {:.2} {volume} {pitch}",
            p[0], p[1], p[2]
        ),
    )
}

// One bolt with its flash: sparks, a crack of thunder, a puff of smoke from the ground.
fn strike(dim: &str, p: [f64; 3], big: bool, snd: &str) -> Vec<Action> {
    let mut v = vec![
        Action::Bolt { dim: dim.to_string(), pos: p, big },
        sparks(dim, p, 0.7, 90),
        sound(dim, p, "minecraft:entity.lightning_bolt.impact", 3.0, 1.0),
        sound(dim, p, "minecraft:item.mace.smash_ground_heavy", 2.0, 0.8),
        sound(dim, p, snd, if big { 6.0 } else { 3.0 }, 1.0),
        at(
            dim,
            format!("particle minecraft:explosion {:.2} {:.2} {:.2} 0.4 0.4 0.4 0 4 force", p[0], p[1] + 0.5, p[2]),
        ),
    ];
    if big {
        v.push(sound(dim, p, "minecraft:entity.lightning_bolt.thunder", 6.0, 0.7));
        v.push(sound(dim, p, "minecraft:entity.generic.explode", 3.0, 0.5));
        v.push(at(
            dim,
            format!("particle minecraft:explosion_emitter {:.2} {:.2} {:.2} 0 0 0 0 1 force", p[0], p[1] + 0.5, p[2]),
        ));
        v.push(sparks(dim, p, 2.5, 260));
    }
    v
}

fn schedule(s: &mut State, delay: u64, actions: Vec<Action>) {
    let due = s.tick + delay;
    let now = Instant::now();
    s.pending.extend(actions.into_iter().map(|a| (due, a, now)));
}

// A hit with Mjolnir landed on a mob at `pos`.
pub fn queue_hit(dim: &str, pos: [f64; 3], origin: [f64; 3], player: &str) {
    with(|s| {
        let mut a = strike(dim, pos, false, "mjolnir:crack");
        a.push(Action::Arc { from: origin, to: [pos[0], pos[1] + 1.0, pos[2]] });
        if safe_name(player) {
            // Thor does not fry himself.
            a.push(Action::Command(format!("effect give {player} minecraft:fire_resistance 3 0 true")));
            a.push(Action::Command(format!("effect give {player} minecraft:resistance 2 4 true")));
        }
        schedule(s, 0, a);
    });
}

// Right click: "Thor's wrath". Returns false while on cooldown.
// The storm darkens the sky, rings the caster in sparks, then marches bolts forward.
pub fn queue_wrath(uuid: u128, dim: &str, player: &str, pos: [f64; 3], yaw_deg: f32) -> bool {
    with(|s| {
        if let Some(last) = s.last_wrath.get(&uuid) {
            if s.tick < last + WRATH_COOLDOWN_TICKS {
                return false;
            }
        }
        s.last_wrath.insert(uuid, s.tick);
        start_cine();
        if safe_name(player) {
            schedule(
                s,
                0,
                vec![
                    Action::Command(format!("effect give {player} minecraft:resistance 8 4 true")),
                    Action::Command(format!("effect give {player} minecraft:fire_resistance 8 0 true")),
                    Action::Command(format!("effect give {player} minecraft:speed 6 1 true")),
                    Action::Command(format!("effect give {player} minecraft:jump_boost 6 1 true")),
                ],
            );
        }
        schedule(
            s,
            0,
            vec![
                Action::Command("weather thunder 15s".to_string()),
                sound(dim, pos, "mjolnir:crack_far", 4.0, 1.0),
                sound(dim, pos, "minecraft:block.respawn_anchor.charge", 4.0, 0.5),
                sound(dim, pos, "minecraft:block.beacon.power_select", 3.0, 0.5),
                sound(dim, pos, "minecraft:entity.lightning_bolt.thunder", 5.0, 0.6),
                sparks(dim, pos, 1.2, 200),
            ],
        );
        let yaw = (yaw_deg as f64).to_radians();
        let dir = [-yaw.sin(), yaw.cos()];
        // The caster is struck first, harmlessly, then the storm walks forward.
        let origin = [pos[0], pos[1] + 1.4, pos[2]];
        schedule(s, 4, strike(dim, pos, true, "mjolnir:blast"));
        for (i, dist) in [5.0_f64, 8.0, 11.0, 14.0].iter().enumerate() {
            let p = [pos[0] + dir[0] * dist, pos[1], pos[2] + dir[1] * dist];
            let delay = 10 + 4 * i as u64;
            schedule(s, delay, strike(dim, p, false, "mjolnir:crack_mid"));
            schedule(s, delay, vec![Action::Arc { from: origin, to: [p[0], p[1] + 1.0, p[2]] }]);
        }
        let end = [pos[0] + dir[0] * 17.0, pos[1], pos[2] + dir[1] * 17.0];
        schedule(s, 30, strike(dim, end, true, "mjolnir:finale"));
        schedule(s, 30, vec![Action::Arc { from: origin, to: [end[0], end[1] + 1.0, end[2]] }]);
        true
    })
}

// Called at the end of each server tick. Returns the actions that are due.
pub fn drain() -> Vec<Action> {
    with(|s| {
        s.tick += 1;
        let now = s.tick;
        let mut due = Vec::new();
        // Actions that waited seconds of real time belong to a world that was left: drop them,
        // so re-entering a world never replays an old storm.
        s.pending.retain(|(t, a, made)| {
            if made.elapsed().as_secs_f32() > 4.0 {
                false
            } else if *t <= now {
                due.push(a.clone());
                false
            } else {
                true
            }
        });
        due
    })
}

// ---- Client-side screen effects -------------------------------------------------------------
// Triggered when a bolt lands. The server and the client share this process in single player and
// for a LAN host, so a plain static carries the trigger; a remote dedicated server would need a
// mod message instead. Timing uses real time, so no client tick hook is needed.
use std::time::Instant;

struct Fx {
    start: Instant,
    pos: [f64; 3],
    strength: f32,
}

static FX: Mutex<Vec<Fx>> = Mutex::new(Vec::new());
const FX_LIFETIME: f32 = 2.5;

pub fn trigger_fx(pos: [f64; 3], big: bool) {
    let mut fx = FX.lock().unwrap_or_else(|e| e.into_inner());
    fx.retain(|f| f.start.elapsed().as_secs_f32() < FX_LIFETIME);
    if fx.len() < 8 {
        fx.push(Fx { start: Instant::now(), pos, strength: if big { 1.0 } else { 0.5 } });
    }
}

fn each_fx<R>(mut f: impl FnMut(&Fx, f32) -> R) -> Vec<R> {
    let fx = FX.lock().unwrap_or_else(|e| e.into_inner());
    fx.iter()
        .filter_map(|x| {
            let age = x.start.elapsed().as_secs_f32();
            (age < FX_LIFETIME).then(|| f(x, age))
        })
        .collect()
}

// Camera shake as [pitch, yaw, roll, offset right, up, back]; Horizon bounds and scales it.
pub fn shake_now() -> [f32; 6] {
    let mut out = [0.0_f32; 6];
    for v in each_fx(|x, t| {
        let a = x.strength * (-t * 2.2).exp();
        [
            a * 2.0 * (t * 47.0).sin(),
            a * 1.5 * (t * 39.0 + 1.0).sin(),
            a * 2.5 * (t * 53.0 + 2.0).sin(),
            a * 0.03 * (t * 43.0).sin(),
            a * 0.05 * (t * 61.0).sin(),
            0.0,
        ]
    }) {
        for i in 0..6 {
            out[i] += v[i];
        }
    }
    out
}

// Full-screen flash as an ARGB colour (alpha 0 when there is none). A big strike opens with three
// hard cuts, white / black / white (about a fifth of a second), then a white-blue flash that fades.
pub fn flash_argb() -> u32 {
    let pack = pack_on();
    let impact = each_fx(|x, t| {
        if pack || x.strength < 1.0 || t >= 0.27 {
            None
        } else if t < 0.09 || t >= 0.18 {
            Some(0xF2FF_FFFFu32)
        } else {
            Some(0xEB00_0000u32)
        }
    })
    .into_iter()
    .flatten()
    .next();
    if let Some(frame) = impact {
        return frame;
    }
    let a = each_fx(|x, t| {
        let k = (1.0 - t / 0.8).max(0.0);
        x.strength * 0.85 * k * k
    })
    .into_iter()
    .fold(0.0_f32, f32::max)
    .clamp(0.0, 0.9);
    ((a * 255.0) as u32) << 24 | 0x00DC_EBFF
}

// Blue-white lights where the bolts landed: (position, intensity), at most four.
pub fn lights_now() -> Vec<([f64; 3], f32)> {
    each_fx(|x, t| ([x.pos[0], x.pos[1] + 1.5, x.pos[2]], (x.strength * (-t * 4.0).exp()).clamp(0.0, 1.0)))
        .into_iter()
        .filter(|(_, i)| *i > 0.02)
        .take(4)
        .collect()
}

// ---- Cinematic: letterbox bars and a slow push-in during Thor's wrath ------------------------
static CINE: Mutex<Option<Instant>> = Mutex::new(None);
const CINE_LEN: f32 = 3.2;

fn start_cine() {
    *CINE.lock().unwrap_or_else(|e| e.into_inner()) = Some(Instant::now());
}

// 0.0 to 1.0: eases in over 0.35 s, holds, eases out over the last 0.7 s.
pub fn cine_amount() -> f32 {
    let t = match *CINE.lock().unwrap_or_else(|e| e.into_inner()) {
        Some(start) => start.elapsed().as_secs_f32(),
        None => return 0.0,
    };
    if t >= CINE_LEN {
        return 0.0;
    }
    let ease = |x: f32| x * x * (3.0 - 2.0 * x);
    let a = (t / 0.35).clamp(0.0, 1.0);
    let b = ((CINE_LEN - t) / 0.7).clamp(0.0, 1.0);
    ease(a.min(b))
}

// Multiplier for the field of view: a slow push-in while the cinematic runs.
pub fn fov_scale() -> f32 {
    1.0 - 0.16 * cine_amount()
}

// While the cinematic runs the game's camera is switched to third person (so the player is drawn) and
// put back afterwards. Codes: 0 first person, 1 third person back, 2 third person front.
static VIEW_SAVED: Mutex<Option<u8>> = Mutex::new(None);

pub fn take_view_change(current: u8) -> Option<u8> {
    let wanted = cine_amount() > 0.001;
    let mut saved = VIEW_SAVED.lock().unwrap_or_else(|e| e.into_inner());
    match (wanted, *saved) {
        (true, None) => {
            *saved = Some(current);
            Some(1)
        }
        (false, Some(old)) => {
            *saved = None;
            Some(old)
        }
        _ => None,
    }
}

// Camera orbit: while the cinematic runs the camera swings around the player and looks at them.
// `eye` and `yaw_deg` are vanilla's camera; returns the position, yaw and pitch to use instead.
pub fn orbit(eye: [f64; 3], yaw_deg: f32) -> Option<([f64; 3], f32, f32)> {
    let amount = cine_amount();
    if amount <= 0.001 {
        return None;
    }
    let t = (*CINE.lock().unwrap_or_else(|e| e.into_inner()))?.elapsed().as_secs_f32();
    let progress = (t / CINE_LEN).clamp(0.0, 1.0) as f64;
    let radius = 5.5 * amount as f64;
    let a = (yaw_deg as f64).to_radians() + 0.25 + progress * 2.4;
    let cam = [eye[0] + a.sin() * radius, eye[1] + 0.3 + 1.3 * amount as f64, eye[2] - a.cos() * radius];
    let d = [eye[0] - cam[0], eye[1] - 0.3 - cam[1], eye[2] - cam[2]];
    let h = (d[0] * d[0] + d[2] * d[2]).sqrt();
    let yaw = (-d[0]).atan2(d[2]).to_degrees() as f32;
    let pitch = (-d[1]).atan2(h).to_degrees() as f32;
    Some((cam, yaw, pitch))
}

// The shader pack asks for its values every frame while it is on, so a recent ask means it is on.
static PACK_SEEN: Mutex<Option<Instant>> = Mutex::new(None);

fn pack_on() -> bool {
    PACK_SEEN
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .map_or(false, |t| t.elapsed().as_secs_f32() < 0.5)
}

// Which hard cut of a big strike's impact frames is showing (0 none, 1 to 3), and the release that
// follows them: 1 as the cuts end, easing to 0 over 0.4 s.
pub fn impact_state() -> (u32, f32) {
    let mut out = (0u32, 0.0_f32);
    for (cut, release) in each_fx(|x, t| {
        if x.strength < 1.0 {
            (0, 0.0)
        } else if t < 0.09 {
            (1, 0.0)
        } else if t < 0.18 {
            (2, 0.0)
        } else if t < 0.27 {
            (3, 0.0)
        } else {
            (0, (1.0 - (t - 0.27) / 0.4).clamp(0.0, 1.0))
        }
    }) {
        if cut > 0 {
            out.0 = cut;
        }
        out.1 = out.1.max(release);
    }
    out
}

// Values for the shader pack (`frame.mods[0]`): x the flash 0..1, y the cinematic 0..1, z the impact
// cut, w the release after the cuts.
pub fn shader_values() -> [f32; 4] {
    *PACK_SEEN.lock().unwrap_or_else(|e| e.into_inner()) = Some(Instant::now());
    let (cut, release) = impact_state();
    [(flash_argb() >> 24) as f32 / 255.0, cine_amount(), cut as f32, release]
}

// ---- Lightning arcs from the hammer to its target -----------------------------------------
struct ArcFx {
    start: Instant,
    from: [f64; 3],
    to: [f64; 3],
    seed: u32,
}

static ARCS: Mutex<Vec<ArcFx>> = Mutex::new(Vec::new());
const ARC_LIFE: f32 = 0.4;

pub fn add_arc(from: [f64; 3], to: [f64; 3]) {
    let mut arcs = ARCS.lock().unwrap_or_else(|e| e.into_inner());
    arcs.retain(|a| a.start.elapsed().as_secs_f32() < ARC_LIFE);
    let seed = (arcs.len() as u32 + 1).wrapping_mul(2654435761) ^ (to[0].to_bits() as u32);
    if arcs.len() < 16 {
        arcs.push(ArcFx { start: Instant::now(), from, to, seed });
    }
}

fn rnd(state: &mut u32) -> f64 {
    *state ^= *state << 13;
    *state ^= *state >> 17;
    *state ^= *state << 5;
    (*state % 20001) as f64 / 10000.0 - 1.0
}

// A jagged path: points along the line pushed sideways, the push shrinking to nothing at both ends.
fn jagged(from: [f64; 3], to: [f64; 3], seed: u32, parts: usize, jitter: f64) -> Vec<[f64; 3]> {
    let mut st = seed | 1;
    (0..=parts)
        .map(|i| {
            let k = i as f64 / parts as f64;
            let taper = (k * (1.0 - k) * 4.0).min(1.0) * jitter;
            let mut p = [0.0; 3];
            for c in 0..3 {
                p[c] = from[c] + (to[c] - from[c]) * k + rnd(&mut st) * taper;
            }
            p
        })
        .collect()
}

// Line segments (from, to, ARGB, width in pixels) for every live arc: a blue glow, a white core and
// two side branches, redrawn with a new shape every 50 ms so the bolt flickers.
pub fn arc_lines() -> Vec<([f64; 3], [f64; 3], u32, f32)> {
    let arcs = ARCS.lock().unwrap_or_else(|e| e.into_inner());
    let mut out = Vec::new();
    for a in arcs.iter() {
        let age = a.start.elapsed().as_secs_f32();
        if age >= ARC_LIFE {
            continue;
        }
        let fade = 1.0 - age / ARC_LIFE;
        let alpha = ((fade * 255.0) as u32).min(255) << 24;
        let len = ((a.to[0] - a.from[0]).powi(2) + (a.to[1] - a.from[1]).powi(2) + (a.to[2] - a.from[2]).powi(2)).sqrt();
        let seed = a.seed ^ ((age / 0.05) as u32).wrapping_mul(40503);
        let path = jagged(a.from, a.to, seed, 12, (0.08 * len).clamp(0.3, 1.6));
        for w in path.windows(2) {
            out.push((w[0], w[1], alpha | 0x0058_A8FF, 9.0));
            out.push((w[0], w[1], alpha | 0x00F2_F8FF, 3.5));
        }
        let mut st = seed.wrapping_add(7) | 1;
        for _ in 0..2 {
            let i = 2 + (rnd(&mut st).abs() * 7.0) as usize;
            let start = path[i.min(path.len() - 1)];
            let end = [start[0] + rnd(&mut st) * 3.0, start[1] + rnd(&mut st) * 2.0, start[2] + rnd(&mut st) * 3.0];
            for w in jagged(start, end, seed.wrapping_add(i as u32), 4, 0.3).windows(2) {
                out.push((w[0], w[1], (alpha / 2 & 0xFF00_0000) | 0x00C8_E4FF, 2.0));
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn reset() {
        with(|s| *s = State::default());
    }

    #[test]
    fn names_are_checked() {
        assert!(safe_name("Thor_1"));
        assert!(!safe_name("a b"));
        assert!(!safe_name("x;stop"));
        assert!(!safe_name(""));
    }

    #[test]
    fn wrath_has_cooldown_and_schedules_bolts() {
        reset();
        assert!(queue_wrath(1, "minecraft:overworld", "Thor", [0.0, 64.0, 0.0], 0.0));
        assert!(!queue_wrath(1, "minecraft:overworld", "Thor", [0.0, 64.0, 0.0], 0.0));
        let mut bolts = 0;
        for _ in 0..40 {
            bolts += drain().iter().filter(|a| matches!(a, Action::Bolt { .. })).count();
        }
        assert_eq!(bolts, 6);
    }

    #[test]
    fn fx_flash_and_shake_fade() {
        assert_eq!(flash_argb() >> 24, 0);
        trigger_fx([0.0, 64.0, 0.0], true);
        assert!(flash_argb() >> 24 > 0);
        assert!(shake_now().iter().any(|v| *v != 0.0));
        assert_eq!(lights_now().len(), 1);
        FX.lock().unwrap().clear();
    }

    #[test]
    fn cinematic_eases() {
        assert_eq!(cine_amount(), 0.0);
        start_cine();
        std::thread::sleep(std::time::Duration::from_millis(500));
        assert!(cine_amount() > 0.9);
        assert!(fov_scale() < 0.9);
        *CINE.lock().unwrap() = None;
    }

    #[test]
    fn arcs_flicker_and_expire() {
        assert!(arc_lines().is_empty());
        add_arc([0.0, 65.0, 0.0], [6.0, 65.0, 2.0]);
        let lines = arc_lines();
        assert!(lines.len() >= 24);
        assert!(lines.iter().all(|l| l.2 >> 24 > 0));
        ARCS.lock().unwrap().clear();
    }

    #[test]
    fn orbit_looks_at_the_player() {
        assert!(orbit([0.0, 65.0, 0.0], 0.0).is_none());
        start_cine();
        std::thread::sleep(std::time::Duration::from_millis(500));
        let (cam, _yaw, pitch) = orbit([0.0, 65.0, 0.0], 0.0).unwrap();
        assert!((cam[0] * cam[0] + cam[2] * cam[2]).sqrt() > 3.0);
        assert!(pitch > 0.0, "camera is above the player, looking down");
        *CINE.lock().unwrap() = None;
    }

    #[test]
    fn hit_is_immediate() {
        reset();
        queue_hit("minecraft:overworld", [1.0, 2.0, 3.0], [0.0, 1.4, 0.0], "Thor");
        assert!(drain().iter().any(|a| matches!(a, Action::Bolt { .. })));
    }
}
