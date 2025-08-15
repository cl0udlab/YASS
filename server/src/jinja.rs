use minijinja::{context, Environment};

pub fn init_templates() -> Environment<'static> {
    let mut env = Environment::new();

    env.add_filter("format_bytes", format_bytes);
    env.add_filter("format_uptime", format_uptime);
    env.add_filter("format_percentage", format_percentage);

    env
}

pub fn render_spa() -> Result<String, Box<dyn std::error::Error>> {
    let spa_template = include_str!("template.html");

    let mut env = init_templates();
    env.add_template("spa", spa_template)?;

    let template = env.get_template("spa")?;
    let rendered = template.render(context! {})?;

    Ok(rendered)
}

fn format_bytes(value: f64) -> Result<String, minijinja::Error> {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
    let mut size = value;
    let mut unit_index = 0;

    while size >= 1024.0 && unit_index < UNITS.len() - 1 {
        size /= 1024.0;
        unit_index += 1;
    }

    Ok(format!("{:.2} {}", size, UNITS[unit_index]))
}

fn format_uptime(seconds: u64) -> Result<String, minijinja::Error> {
    let days = seconds / 86400;
    let hours = (seconds % 86400) / 3600;
    let minutes = (seconds % 3600) / 60;

    if days > 0 {
        Ok(format!("{}天 {}小時 {}分鐘", days, hours, minutes))
    } else if hours > 0 {
        Ok(format!("{}小時 {}分鐘", hours, minutes))
    } else {
        Ok(format!("{}分鐘", minutes))
    }
}

fn format_percentage(value: f64) -> Result<String, minijinja::Error> {
    Ok(format!("{:.1}%", value))
}
