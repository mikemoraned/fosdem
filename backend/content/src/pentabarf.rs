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
    #[xmlserde(name = b"id", ty = "attr")]
    pub id: u32,
    #[xmlserde(name = b"start", ty = "child")]
    pub start: Text,
    #[xmlserde(name = b"duration", ty = "child")]
    pub duration: Text,
    #[xmlserde(name = b"title", ty = "child")]
    pub title: Text,
    #[xmlserde(name = b"slug", ty = "child")]
    pub slug: Text,
    #[xmlserde(name = b"url", ty = "child")]
    pub url: Text,
    #[xmlserde(name = b"abstract", ty = "child")]
    pub r#abstract: Text,
}

#[derive(XmlDeserialize, Default, Debug)]
pub struct Text {
    #[xmlserde(ty = "text")]
    pub value: String,
}
