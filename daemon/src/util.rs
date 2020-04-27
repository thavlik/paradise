pub fn get_host_by_name(name: &str) -> Result<cpal::Host, anyhow::Error> {
    let available_hosts = cpal::available_hosts();
    for host_id in available_hosts {
        if host_id.name() == name {
            return Ok(cpal::host_from_id(host_id)?);
        }
    }
    Err(anyhow::Error::msg(format!("host \"{}\" not found", name)))
}
