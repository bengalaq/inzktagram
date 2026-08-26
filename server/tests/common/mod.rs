use feed_core::{Candidate, FeedInput, Params, UserConfig};

pub fn sample_input(n: u64, alg: u8) -> FeedInput {
    let candidates = (0..n)
        .map(|i| {
            let r = (i
                .wrapping_mul(6_364_136_223_846_793_005)
                .wrapping_add(144_115_188))
                >> 33;
            Candidate {
                post_id: i + 1,
                author_id: (i % 7) + 1,
                created_at: 1_756_000_000 - ((r % 2_880) + 1) * 60,
                likes: (r % 900) as u32,
                comments: (r % 90) as u32,
                length_chars: 80 + ((r % 11) * 60) as u32,
                is_followed: i % 3 != 0,
            }
        })
        .collect();
    FeedInput {
        config: UserConfig {
            user_id: 42,
            algorithm_id: alg,
            nonce: 7,
        },
        params: Params::default(),
        candidates,
        now: 1_756_000_000,
    }
}
