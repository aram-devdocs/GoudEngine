use super::*;

#[test]
fn test_fog_config_mode_int() {
    let exp = FogConfig {
        enabled: true,
        color: Vector3::new(0.5, 0.5, 0.5),
        mode: FogMode::Exponential { density: 0.05 },
    };
    assert_eq!(exp.mode_int(), 0);

    let lin = FogConfig {
        enabled: true,
        color: Vector3::new(0.5, 0.5, 0.5),
        mode: FogMode::Linear {
            start: 10.0,
            end: 100.0,
        },
    };
    assert_eq!(lin.mode_int(), 1);
}

#[test]
fn test_fog_config_density() {
    let exp = FogConfig {
        enabled: true,
        color: Vector3::new(0.0, 0.0, 0.0),
        mode: FogMode::Exponential { density: 0.03 },
    };
    assert!((exp.density() - 0.03).abs() < f32::EPSILON);

    let lin = FogConfig {
        enabled: true,
        color: Vector3::new(0.0, 0.0, 0.0),
        mode: FogMode::Linear {
            start: 10.0,
            end: 100.0,
        },
    };
    assert!((lin.density() - 0.0).abs() < f32::EPSILON);
}

#[test]
fn test_fog_config_start() {
    let exp = FogConfig {
        enabled: true,
        color: Vector3::new(0.0, 0.0, 0.0),
        mode: FogMode::Exponential { density: 0.02 },
    };
    assert!((exp.start() - 0.0).abs() < f32::EPSILON);

    let lin = FogConfig {
        enabled: true,
        color: Vector3::new(0.0, 0.0, 0.0),
        mode: FogMode::Linear {
            start: 80.0,
            end: 200.0,
        },
    };
    assert!((lin.start() - 80.0).abs() < f32::EPSILON);
}

#[test]
fn test_fog_config_end() {
    let exp = FogConfig {
        enabled: true,
        color: Vector3::new(0.0, 0.0, 0.0),
        mode: FogMode::Exponential { density: 0.02 },
    };
    assert!((exp.end() - 0.0).abs() < f32::EPSILON);

    let lin = FogConfig {
        enabled: true,
        color: Vector3::new(0.0, 0.0, 0.0),
        mode: FogMode::Linear {
            start: 80.0,
            end: 200.0,
        },
    };
    assert!((lin.end() - 200.0).abs() < f32::EPSILON);
}

#[test]
fn test_fog_config_default_is_exponential() {
    let cfg = FogConfig::default();
    assert_eq!(cfg.mode_int(), 0);
    assert!(matches!(cfg.mode, FogMode::Exponential { .. }));
}
