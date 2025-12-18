use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn test_version() {
    let mut cmd = Command::cargo_bin("be").expect("No se encontro el binario 'be'");
    cmd.arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains("be 1.0."));
}

#[test]
fn test_help() {
    let mut cmd = Command::cargo_bin("be").expect("No se encontro el binario 'be'");
    cmd.arg("help")
        .assert()
        .success()
        .stdout(predicate::str::contains("MANUAL DE USUARIO"));
}

#[test]
fn test_status_fails_clean() {
    let mut cmd = Command::cargo_bin("be").expect("No se encontro el binario 'be'");
    cmd.arg("status")
        .assert()
        .success() // Deberia salir con 0, pero mostrar error en output
        .stdout(
            predicate::str::contains("No se encontro %LOCALAPPDATA%")
                .or(predicate::str::contains("Hay inconsistencias")),
        );
}
