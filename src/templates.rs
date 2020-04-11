use yarte::Template;

#[derive(Template)]
#[template(path = "pages/register.hbs")]
pub struct Register {
    pub sent: bool,
    pub error: Option<String>
}
