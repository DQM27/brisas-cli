use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn test_version() {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_be"));
    cmd.arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains("be 1.0."));
}

#[test]
fn test_help() {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_be"));
    cmd.arg("help")
        .assert()
        .success()
        .stdout(predicate::str::contains("MANUAL DE USUARIO"));
}

#[test]
fn test_status_fails_clean() {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_be"));
    cmd.env("LOCALAPPDATA", "C:\\FakePathThatDoesNotExist")
        .arg("status")
        .assert()
        .success() // Deberia salir con 0, pero mostrar error en output
        .stdout(
            predicate::str::contains("No se encontro %LOCALAPPDATA%")
                .or(predicate::str::contains("Hay inconsistencias"))
                .or(predicate::str::contains("No encontrado")),
        );
}
