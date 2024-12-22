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
    #[xmlserde(name = b"track", ty = "child")]
    pub track: Text,
    #[xmlserde(name = b"title", ty = "child")]
    pub title: Text,
    #[xmlserde(name = b"persons", ty = "child")]
    pub persons: Persons,
    #[xmlserde(name = b"slug", ty = "child")]
    pub slug: Text,
    #[xmlserde(name = b"url", ty = "child")]
    pub url: Text,
    #[xmlserde(name = b"abstract", ty = "child")]
    pub r#abstract: Text,
    #[xmlserde(name = b"attachments", ty = "child")]
    pub attachments: Attachments,
    #[xmlserde(name = b"links", ty = "child")]
    pub links: Links,
}

#[derive(XmlDeserialize, Default, Debug)]
pub struct Text {
    #[xmlserde(ty = "text")]
    pub value: String,
}

#[derive(XmlDeserialize, Default, Debug)]
pub struct Persons {
    #[xmlserde(name = b"person", ty = "child")]
    pub persons: Vec<Person>,
}

#[derive(XmlDeserialize, Default, Debug)]
pub struct Person {
    #[xmlserde(name = b"id", ty = "attr")]
    pub id: u32,
    #[xmlserde(ty = "text")]
    pub name: String,
}

#[derive(XmlDeserialize, Default, Debug)]
pub struct Attachments {
    #[xmlserde(name = b"attachment", ty = "child")]
    pub attachments: Vec<Attachment>,
}

#[derive(XmlDeserialize, Default, Debug)]
pub struct Attachment {
    #[xmlserde(name = b"type", ty = "attr")]
    pub r#type: String,
    #[xmlserde(name = b"href", ty = "attr")]
    pub href: String,
}

#[derive(XmlDeserialize, Default, Debug)]
pub struct Links {
    #[xmlserde(name = b"link", ty = "child")]
    pub links: Vec<Link>,
}

#[derive(XmlDeserialize, Default, Debug)]
pub struct Link {
    #[xmlserde(name = b"href", ty = "attr")]
    pub href: String,
    #[xmlserde(ty = "text")]
    pub name: String,
}
