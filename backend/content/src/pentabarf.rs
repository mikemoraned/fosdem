use xmlserde_derives::XmlDeserialize;

#[derive(XmlDeserialize, Default, Debug)]
#[xmlserde(root = b"schedule")]
pub struct Schedule {
    #[xmlserde(name = b"day", ty = "child")]
    pub days: Vec<Day>,
}

#[derive(XmlDeserialize, Default, Debug)]
pub struct Day {
    #[xmlserde(name = b"date", ty = "attr")]
    pub date: String,
    #[xmlserde(name = b"room", ty = "child")]
    pub rooms: Vec<Room>,
}

#[derive(XmlDeserialize, Default, Debug)]
pub struct Room {
    #[xmlserde(name = b"name", ty = "attr")]
    pub name: String,
    #[xmlserde(name = b"event", ty = "child")]
    pub events: Vec<crate::pentabarf::Event>,
}

#[derive(XmlDeserialize, Default, Debug)]
pub struct Event {
    #[xmlserde(name = b"title", ty = "child")]
    pub title: Title,
    #[xmlserde(name = b"slug", ty = "child")]
    pub slug: Abstract,
    #[xmlserde(name = b"abstract", ty = "child")]
    pub r#abstract: Abstract,
}

#[derive(XmlDeserialize, Default, Debug)]
pub struct Abstract {
    #[xmlserde(ty = "text")]
    pub value: String,
}

#[derive(XmlDeserialize, Default, Debug)]
pub struct Title {
    #[xmlserde(ty = "text")]
    pub value: String,
}

#[derive(XmlDeserialize, Default, Debug)]
pub struct Slug {
    #[xmlserde(ty = "text")]
    pub value: String,
}
