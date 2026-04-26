use std::ops::Deref;

use you_shall_not_pass::password_generator::{
    DEFAULTS, Defaults, PasswordError, calc_entropy, generate_password,
};

// ---------------------------------------------------------------------------
// Task 1: Happy-path tests
// ---------------------------------------------------------------------------

#[test]
fn test_generate_password_default_length() {
    let password = generate_password(&DEFAULTS, rand::rng()).unwrap();
    assert_eq!(password.len(), DEFAULTS.length);
}

#[test]
fn test_generate_password_custom_length() {
    let config = Defaults {
        length: 24,
        ..DEFAULTS
    };
    let password = generate_password(&config, rand::rng()).unwrap();
    assert_eq!(password.len(), 24);
}

#[test]
fn test_generate_password_no_symbols() {
    let config = Defaults {
        length: 16,
        symbol_flag: false,
        ..DEFAULTS
    };
    let password = generate_password(&config, rand::rng()).unwrap();
    assert_eq!(password.len(), 16);
    for ch in password.chars() {
        assert!(
            !"!@#$%^&*".contains(ch),
            "found disallowed symbol character: {ch}"
        );
    }
}

#[test]
fn test_calc_entropy_classical() {
    let config = Defaults {
        quantum_check: false,
        ..DEFAULTS
    };
    let entropy = calc_entropy(&config, rand::rng()).unwrap();
    assert!(
        entropy > 60.0,
        "classical entropy {entropy} should be > 60.0"
    );
}

#[test]
fn test_calc_entropy_quantum() {
    let classical_config = Defaults {
        quantum_check: false,
        ..DEFAULTS
    };
    let quantum_config = Defaults {
        quantum_check: true,
        ..DEFAULTS
    };
    let classical = calc_entropy(&classical_config, rand::rng()).unwrap();
    let quantum = calc_entropy(&quantum_config, rand::rng()).unwrap();
    let expected = classical / 2.0;
    let diff = (quantum - expected).abs();
    assert!(
        diff < 1.0,
        "quantum entropy {quantum} should be ~half of classical {classical} (expected {expected}, diff {diff})"
    );
}

#[test]
fn test_password_is_zeroizing() {
    let password = generate_password(&DEFAULTS, rand::rng()).unwrap();
    // Verify Zeroizing<String> implements Deref<Target = String>
    let s: &String = password.deref();
    assert_eq!(s.len(), DEFAULTS.length);
    // Also accessible as &str via auto-deref coercion
    let s2: &str = &password;
    assert_eq!(s2.len(), DEFAULTS.length);
}

// ---------------------------------------------------------------------------
// Task 2: Error-path tests
// ---------------------------------------------------------------------------

#[test]
fn test_empty_length_error() {
    let config = Defaults {
        length: 0,
        ..DEFAULTS
    };
    let err = generate_password(&config, rand::rng()).unwrap_err();
    assert!(matches!(err, PasswordError::EmptyLength));
}

#[test]
fn test_no_charset_groups_error() {
    let config = Defaults {
        length: 12,
        alphabet_flag: false,
        caps_alphabet_flag: false,
        num_flag: false,
        symbol_flag: false,
        ..DEFAULTS
    };
    let err = generate_password(&config, rand::rng()).unwrap_err();
    assert!(matches!(err, PasswordError::NoCharsetGroups));
}

#[test]
fn test_length_too_short_error() {
    let config = Defaults {
        length: 2,
        ..DEFAULTS
    };
    let err = generate_password(&config, rand::rng()).unwrap_err();
    assert!(
        matches!(
            err,
            PasswordError::LengthTooShort {
                required: 4,
                actual: 2
            }
        ),
        "expected LengthTooShort(required=4, actual=2), got {err:?}"
    );
}

#[test]
fn test_calc_entropy_empty_length_error() {
    let config = Defaults {
        length: 0,
        ..DEFAULTS
    };
    let err = calc_entropy(&config, rand::rng()).unwrap_err();
    assert!(matches!(err, PasswordError::EmptyLength));
}

#[test]
fn test_calc_entropy_no_charset_error() {
    let config = Defaults {
        length: 12,
        alphabet_flag: false,
        caps_alphabet_flag: false,
        num_flag: false,
        symbol_flag: false,
        ..DEFAULTS
    };
    let err = calc_entropy(&config, rand::rng()).unwrap_err();
    assert!(matches!(err, PasswordError::NoCharsetGroups));
}

#[test]
fn test_calc_entropy_length_too_short() {
    let config = Defaults {
        length: 2,
        ..DEFAULTS
    };
    let err = calc_entropy(&config, rand::rng()).unwrap_err();
    assert!(
        matches!(
            err,
            PasswordError::LengthTooShort {
                required: 4,
                actual: 2
            }
        ),
        "expected LengthTooShort(required=4, actual=2), got {err:?}"
    );
}

// ---------------------------------------------------------------------------
// Task 2: Entropy validation alignment tests
// ---------------------------------------------------------------------------

#[test]
fn test_entropy_increases_with_length() {
    let short_config = Defaults {
        length: 8,
        quantum_check: false,
        ..DEFAULTS
    };
    let long_config = Defaults {
        length: 16,
        quantum_check: false,
        ..DEFAULTS
    };
    let short_entropy = calc_entropy(&short_config, rand::rng()).unwrap();
    let long_entropy = calc_entropy(&long_config, rand::rng()).unwrap();
    assert!(
        long_entropy > short_entropy,
        "entropy at length 16 ({long_entropy}) should be > entropy at length 8 ({short_entropy})"
    );
}

#[test]
fn test_entropy_decreases_with_fewer_groups() {
    let all_config = Defaults {
        quantum_check: false,
        ..DEFAULTS
    };
    let few_config = Defaults {
        caps_alphabet_flag: false,
        num_flag: false,
        symbol_flag: false,
        quantum_check: false,
        ..DEFAULTS
    };
    let all_entropy = calc_entropy(&all_config, rand::rng()).unwrap();
    let few_entropy = calc_entropy(&few_config, rand::rng()).unwrap();
    assert!(
        all_entropy > few_entropy,
        "entropy with all groups ({all_entropy}) should be > entropy with only lowercase ({few_entropy})"
    );
}
