//! Structural check that the shipped UI exposes link input and download controls.

#[test]
fn ui_has_link_input_and_download_control() {
    let html = include_str!("../../src/index.html");
    assert!(
        html.contains("id=\"links\""),
        "UI must include a control to enter Instagram link(s)"
    );
    assert!(
        html.contains("<textarea"),
        "link control should be a textarea for one or many Instagram links"
    );
    assert!(
        html.contains("id=\"download\""),
        "UI must include a control to start download"
    );
    assert!(
        html.contains("id=\"destination\""),
        "UI must include a destination control"
    );
}

#[test]
fn tauri_mac_bundle_is_configured() {
    let conf = include_str!("../tauri.conf.json");
    assert!(conf.contains("\"identifier\": \"com.ins.downloader\""));
    assert!(conf.contains("\"frontendDist\": \"../src\""));
    assert!(
        conf.contains("\"targets\"") && (conf.contains("app") || conf.contains("all")),
        "mac bundle target missing"
    );
}
