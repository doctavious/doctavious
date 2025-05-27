
// TODO: sub-commands


pub struct CmdOption {
    pub name: String,
    pub short_hand: Option<String>,
    pub default_value: Option<String>,
    pub value_type: Option<String>,
    pub required: bool,
    pub usage: Option<String>
    // global?
}

pub struct CmdArguments {

}

pub struct GlobalOptions {
    options: Vec<CmdOption>,
}

pub struct CmdDoc {
    pub name: String,
    pub synopsis: Option<String>,
    pub description: Option<String>,
    pub usage: Option<String>,
    pub options: Vec<cmdOption>,
    pub inherited_options: Vec<CmdOption>,
    pub example: Option<String>,
    pub see_also: Vec<String>
}
