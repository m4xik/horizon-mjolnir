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
}

#[derive(Default)]
struct State {
    tick: u64,
    pending: Vec<(u64, Action)>,
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
fn strike(dim: &str, p: [f64; 3], big: bool) -> Vec<Action> {
    let mut v = vec![
        Action::Bolt { dim: dim.to_string(), pos: p, big },
        sparks(dim, p, 0.7, 90),
        sound(dim, p, "minecraft:entity.lightning_bolt.impact", 3.0, 1.0),
        at(
            dim,
            format!("particle minecraft:explosion {:.2} {:.2} {:.2} 0.4 0.4 0.4 0 4 force", p[0], p[1] + 0.5, p[2]),
        ),
    ];
    if big {
        v.push(sound(dim, p, "minecraft:entity.lightning_bolt.thunder", 6.0, 0.7));
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
    s.pending.extend(actions.into_iter().map(|a| (due, a)));
}

// A hit with Mjolnir landed on a mob at `pos`.
pub fn queue_hit(dim: &str, pos: [f64; 3], player: &str) {
    with(|s| {
        let mut a = strike(dim, pos, false);
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
                sound(dim, pos, "minecraft:entity.lightning_bolt.thunder", 5.0, 0.6),
                sparks(dim, pos, 1.2, 200),
            ],
        );
        let yaw = (yaw_deg as f64).to_radians();
        let dir = [-yaw.sin(), yaw.cos()];
        // The caster is struck first, harmlessly, then the storm walks forward.
        schedule(s, 4, strike(dim, pos, true));
        for (i, dist) in [5.0_f64, 8.0, 11.0, 14.0].iter().enumerate() {
            let p = [pos[0] + dir[0] * dist, pos[1], pos[2] + dir[1] * dist];
            schedule(s, 10 + 4 * i as u64, strike(dim, p, false));
        }
        let end = [pos[0] + dir[0] * 17.0, pos[1], pos[2] + dir[1] * 17.0];
        schedule(s, 30, strike(dim, end, true));
        true
    })
}

// Called at the end of each server tick. Returns the actions that are due.
pub fn drain() -> Vec<Action> {
    with(|s| {
        s.tick += 1;
        let now = s.tick;
        let mut due = Vec::new();
        s.pending.retain(|(t, a)| {
            if *t <= now {
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
    let impact = each_fx(|x, t| {
        if x.strength < 1.0 || t >= 0.21 {
            None
        } else if t < 0.07 || t >= 0.14 {
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
        let k = (1.0 - t / 0.45).max(0.0);
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
    fn hit_is_immediate() {
        reset();
        queue_hit("minecraft:overworld", [1.0, 2.0, 3.0], "Thor");
        assert!(drain().iter().any(|a| matches!(a, Action::Bolt { .. })));
    }
}
