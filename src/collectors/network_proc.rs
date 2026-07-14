use std::path::Path;

const TCP_ESTABLISHED: &str = "01";

pub struct NetworkProcCollector;

impl NetworkProcCollector {
    pub fn active_connection_count() -> anyhow::Result<usize> {
        active_connection_count_from_path("/proc/net/tcp")
    }
}

pub fn active_connection_count_from_path(path: impl AsRef<Path>) -> anyhow::Result<usize> {
    let contents = std::fs::read_to_string(path)?;
    Ok(parse_active_connection_count(&contents))
}

pub fn parse_active_connection_count(contents: &str) -> usize {
    contents
        .lines()
        .skip(1)
        .filter_map(parse_tcp_state)
        .filter(|state| *state == TCP_ESTABLISHED)
        .count()
}

fn parse_tcp_state(line: &str) -> Option<&str> {
    line.split_whitespace().nth(3)
}

#[cfg(test)]
mod tests {
    use super::parse_active_connection_count;

    #[test]
    fn counts_only_established_tcp_connections() {
        let proc_net_tcp = r#"
  sl  local_address rem_address   st tx_queue rx_queue tr tm->when retrnsmt   uid  timeout inode
   0: 0100007F:1F90 0100007F:CAFE 01 00000000:00000000 00:00000000 00000000  501        0 12345 1 0000000000000000 20 4 30 10 -1
   1: 0100007F:1F91 00000000:0000 0A 00000000:00000000 00:00000000 00000000  501        0 12346 1 0000000000000000 20 4 30 10 -1
   2: 0100007F:1F92 0100007F:BEEF 01 00000000:00000000 00:00000000 00000000  501        0 12347 1 0000000000000000 20 4 30 10 -1
"#;

        assert_eq!(parse_active_connection_count(proc_net_tcp), 2);
    }

    #[test]
    fn ignores_malformed_rows() {
        let proc_net_tcp = r#"
  sl  local_address rem_address   st tx_queue rx_queue tr tm->when retrnsmt   uid  timeout inode
malformed
   0: 0100007F:1F90 0100007F:CAFE 01 00000000:00000000 00:00000000
"#;

        assert_eq!(parse_active_connection_count(proc_net_tcp), 1);
    }
}
