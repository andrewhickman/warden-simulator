// use rand::Rng;
// use wdn_rl_ppo::Environment;
// use wdn_rl_model::Step;

// pub struct WdnEnvironment {
//     app: App,
// }

// impl Default for WdnEnvironment {
//     fn default() -> Self {
//         Self {

//         }
//     }
// }

// impl Environment for WdnEnvironment {
//     type Action = ();
//     type Observation = ();

//     fn reset<R: Rng>(&mut self, rng: &mut R) -> Self::Observation {
//         ()
//     }

//     fn step(&mut self, action: Self::Action) -> Step<Self::Observation, Self::Action> {
//         Step {
//             observation: (),
//             action,
//             reward: 0.0,
//             terminated: false,
//             truncated: false,
//         }
//     }
// }

fn main() {
    println!("Hello, world!");
}
