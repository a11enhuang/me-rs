use log4rs::config::RawConfig;

pub fn default() {
    let default_config = "
    appenders:
      stdout:
        kind: console
        encoder:
          pattern: \"{d(%Y-%m-%d %H:%M:%S)} {h({l})} {pid} --- [{T}][{X(requestId)}] {h({f})} : {m}{n}\"
    root:
      level: info
      appenders:
        - stdout
    ";
    let config: RawConfig = serde_yaml::from_str(default_config).unwrap();
    log4rs::init_raw_config(config).unwrap();
    // log4rs::init_file("resources/log4rs.yaml", Default::default()).unwrap();
}