// id INTEGER NOT NULL PRIMARY KEY,
// type TEXT NOT NULL,
// dateCreated INTEGER /* datetime */ NOT NULL,
// dateModified INTEGER /* datetime */ NOT NULL,
// dateServerModified INTEGER /* datetime */ NOT NULL,
// deleted INTEGER /* boolean */ NOT NULL DEFAULT 0,
// version INTEGER NOT NULL

pub struct Item {
    pub id: i64,
    pub _type: String,
    pub date_created: i64,
    pub date_modified: i64,
    pub date_server_modified: i64,
    pub deleted: i8,
    pub version: i64,
}
