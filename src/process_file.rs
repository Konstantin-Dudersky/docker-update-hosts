use crate::{
    constants::{BEGIN_BLOCK, END_BLOCK},
    docker_host::DockerHost,
};

/// Определить позиции начала и конца вставки
fn find_positions(input: &[String]) -> Option<(usize, usize)> {
    let begin_position = input.iter().position(|l| *l == BEGIN_BLOCK)?;
    let end_position = input.iter().position(|l| *l == END_BLOCK)?;
    Some((begin_position, end_position))
}

fn copy_hosts_file(source_file: &[&str]) -> Vec<String> {
    source_file
        .iter()
        .map(|l| l.to_string())
        .collect::<Vec<String>>()
}

fn delete_hosts_from_file(file: &mut Vec<String>, begin: usize, end: usize) {
    file.drain(begin..=end);
}

fn append_hosts_to_file(file: &mut Vec<String>, docker_hosts: &[DockerHost]) {
    file.push(BEGIN_BLOCK.to_string());
    for dh in docker_hosts {
        file.push(dh.into_file_line())
    }
    file.push(END_BLOCK.to_string());
}

pub fn process_hosts_file(source_file: &[&str], docker_hosts: &[DockerHost]) -> Vec<String> {
    let mut new_file = copy_hosts_file(source_file);

    let positions = find_positions(&new_file);

    match positions {
        Some((begin, end)) => delete_hosts_from_file(&mut new_file, begin, end),
        None => {}
    }
    append_hosts_to_file(&mut new_file, docker_hosts);
    new_file
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_addition() {
        let input_file = vec!["a", "b", "c"];
        let dh = vec![
            DockerHost {
                hostname: "host1".into(),
                ip_address: "1.2.3.4".into(),
            },
            DockerHost {
                hostname: "host2".into(),
                ip_address: "1.2.3.4".into(),
            },
        ];

        let output_file = process_hosts_file(&input_file, &dh);
        assert_eq!(
            output_file,
            vec![
                "a",
                "b",
                "c",
                BEGIN_BLOCK,
                "1.2.3.4         host1",
                "1.2.3.4         host2",
                END_BLOCK
            ]
        );
    }

    #[test]
    fn test_replace() {
        let input_file = vec![
            "a",
            "b",
            "c",
            BEGIN_BLOCK,
            "1.1.1.1         host1",
            "2.2.2.2         host2",
            END_BLOCK,
            "d",
            "e",
        ];
        let dh = vec![
            DockerHost {
                hostname: "host3".into(),
                ip_address: "3.3.3.3".into(),
            },
            DockerHost {
                hostname: "host4".into(),
                ip_address: "4.4.4.4".into(),
            },
        ];

        let output_file = process_hosts_file(&input_file, &dh);
        assert_eq!(
            output_file,
            vec![
                "a",
                "b",
                "c",
                "d",
                "e",
                BEGIN_BLOCK,
                "3.3.3.3         host3",
                "4.4.4.4         host4",
                END_BLOCK
            ]
        );
    }
}
