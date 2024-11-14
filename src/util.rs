use rand::seq::IteratorRandom;
use teloxide::types::{Rgb, User};

pub fn get_random_topic_color() -> Rgb {
    let rnd = vec![0x6FB9F0, 0xFFD67E, 0xCB86DB, 0x8EEE98, 0xFF93B2, 0xFB6F5F]
        .into_iter()
        .choose(&mut rand::thread_rng())
        .unwrap();

    Rgb::from_u32(rnd)
}

pub fn get_user_name(user: &User) -> String {
    user.username.clone().unwrap_or(user.first_name.clone())
}
