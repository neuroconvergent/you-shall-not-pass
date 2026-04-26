// Copyright 2026 Sundar Gurumurthy
// SPDX-License-Identifier: BSD-3-Clause-No-Military-License

pub mod password_generator {
    use std::collections::HashSet;

    use rand::{Rng, RngCore, seq::SliceRandom};
    use thiserror::Error;
    use zeroize::Zeroizing;

    #[derive(Debug, Clone)]
    pub struct Defaults {
        pub length: usize,
        pub charset_groups: &'static [&'static str],
        pub target_entropy: f64,
        pub chars_to_avoid: &'static str,
        pub utf8_flag: bool,
        pub alphabet_flag: bool,
        pub caps_alphabet_flag: bool,
        pub num_flag: bool,
        pub symbol_flag: bool,
        pub quantum_check: bool,
    }

    pub const DEFAULTS: Defaults = Defaults {
        length: 12,
        charset_groups: &[
            "abcdefghijklmnopqrstuvwxyz",
            "ABCDEFGHIJKLMNOPQRSTUVWXYZ",
            "0123456789",
            "!@#$%^&*",
        ],
        target_entropy: 80.0,
        chars_to_avoid: "",
        utf8_flag: false,
        alphabet_flag: true,
        caps_alphabet_flag: true,
        num_flag: true,
        symbol_flag: true,
        quantum_check: true,
    };

    #[derive(Debug, Error)]
    pub enum PasswordError {
        #[error("password length must be greater than zero")]
        EmptyLength,
        #[error("no charset groups enabled for generation")]
        NoCharsetGroups,
        #[error("password length {actual} is shorter than required minimum {required}")]
        LengthTooShort { required: usize, actual: usize },
        #[error("character group at index {group_index} has no usable characters after filtering")]
        ExhaustedGroup { group_index: usize },
        #[error("character pool is empty after applying filters")]
        EmptyCharacterPool,
    }

    type Result<T> = std::result::Result<T, PasswordError>;

    pub fn generate_password(config: &Defaults, mut rng: impl Rng) -> Result<Zeroizing<String>> {
        if config.length == 0 {
            return Err(PasswordError::EmptyLength);
        }

        let groups = enabled_groups(config);
        if groups.is_empty() {
            return Err(PasswordError::NoCharsetGroups);
        }

        if config.length < groups.len() {
            return Err(PasswordError::LengthTooShort {
                required: groups.len(),
                actual: config.length,
            });
        }

        let avoid = build_avoid_set(config);
        let filtered_groups = filtered_groups(config, &groups, &avoid)?;
        let pool = build_character_pool(&filtered_groups);

        if pool.is_empty() {
            return Err(PasswordError::EmptyCharacterPool);
        }

        let mut picks = Zeroizing::new(Vec::with_capacity(config.length));

        for chars in filtered_groups.iter() {
            let selection = chars[random_index(chars.len(), &mut rng)];
            picks.push(selection);
        }

        while picks.len() < config.length {
            let selection = pool[random_index(pool.len(), &mut rng)];
            picks.push(selection);
        }

        picks.as_mut_slice().shuffle(&mut rng);

        let password: String = picks.iter().copied().collect();
        Ok(Zeroizing::new(password))
    }

    pub fn calc_entropy(config: &Defaults, mut rng: impl Rng) -> Result<f64> {
        let _ = &mut rng;
        if config.length == 0 {
            return Err(PasswordError::EmptyLength);
        }

        let groups = enabled_groups(config);
        if groups.is_empty() {
            return Err(PasswordError::NoCharsetGroups);
        }

        if config.length < groups.len() {
            return Err(PasswordError::LengthTooShort {
                required: groups.len(),
                actual: config.length,
            });
        }

        let avoid = build_avoid_set(config);
        let filtered_groups = filtered_groups(config, &groups, &avoid)?;
        let pool = build_character_pool(&filtered_groups);

        if pool.is_empty() {
            return Err(PasswordError::EmptyCharacterPool);
        }

        let mut entropy = (pool.len() as f64).log2() * config.length as f64;
        if config.quantum_check {
            entropy /= 2.0;
        }

        Ok(entropy)
    }

    fn enabled_groups(config: &Defaults) -> Vec<&str> {
        let mut groups = Vec::new();
        for (index, group) in config.charset_groups.iter().enumerate() {
            let include = match index {
                0 => config.alphabet_flag,
                1 => config.caps_alphabet_flag,
                2 => config.num_flag,
                3 => config.symbol_flag,
                _ => true,
            };

            if include {
                groups.push(*group);
            }
        }

        groups
    }

    fn build_avoid_set(config: &Defaults) -> HashSet<char> {
        config.chars_to_avoid.chars().collect()
    }

    fn filtered_groups(
        config: &Defaults,
        groups: &[&str],
        avoid: &HashSet<char>,
    ) -> Result<Vec<Vec<char>>> {
        let mut filtered = Vec::with_capacity(groups.len());

        for (index, group) in groups.iter().enumerate() {
            let chars: Vec<char> = group
                .chars()
                .filter(|ch| (config.utf8_flag || ch.is_ascii()) && !avoid.contains(ch))
                .collect();

            if chars.is_empty() {
                return Err(PasswordError::ExhaustedGroup { group_index: index });
            }

            filtered.push(chars);
        }

        Ok(filtered)
    }

    fn build_character_pool(groups: &[Vec<char>]) -> Vec<char> {
        let mut pool = Vec::new();

        for chars in groups {
            for ch in chars {
                if !pool.contains(ch) {
                    pool.push(*ch);
                }
            }
        }

        pool
    }

    fn random_index(len: usize, rng: &mut impl RngCore) -> usize {
        debug_assert!(len > 0);

        let range = len as u64;
        let cutoff = u64::MAX - (u64::MAX % range);

        loop {
            let sample = rng.next_u64();
            if sample < cutoff {
                return (sample % range) as usize;
            }
        }
    }
}
