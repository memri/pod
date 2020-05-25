pub fn get_edge_props() -> [&'static str; 23] {
    let edge_props: [&str; 23] = [
        "addresses: [uid] ",
        "appliesTo: [uid] ",
        "auditTarget: uid ",
        "changelog: [uid] ",
        "comments: [uid] ",
        "companies: [uid] ",
        "country: uid ",
        "diets: [uid] ",
        "file: uid ",
        "flag: uid ",
        "includes: [uid] ",
        "labels: [uid] ",
        "location: uid ",
        "medicalConditions: [uid] ",
        "onlineProfiles: [uid] ",
        "phoneNumbers: [uid] ",
        "profilePicture: uid ",
        "publicKeys: [uid] ",
        "relations: [uid] ",
        "sharedWith: [uid] ",
        "usedBy: [uid] ",
        "websites: [uid] ",
        "writtenBy: [uid] ",
    ];
    edge_props
}

pub fn get_string_props() -> Vec<&'static str> {
    let string_props: Vec<&str> = vec![
        "action",
        "city",
        "color",
        "comment",
        "computeTitle",
        "content",
        "contents",
        "firstName",
        "gender",
        "genericType",
        "handle",
        "key",
        "lastName",
        "name",
        "number",
        "postalCode",
        "sexualOrientation",
        "state",
        "street",
        "title",
        "type",
        "uri",
        "url",
    ];
    string_props
}

pub fn get_other_props() -> Vec<&'static str> {
    let other_props: Vec<&str> = vec![
        "additions: [string] @index(term) .",
        "age: float @index(float) .",
        "armLength: float @index(float) .",
        "birthDate: datetime .",
        "bitrate: int @index(int) .",
        "date: datetime .",
        "dateAccessed: datetime .",
        "dateCreated: datetime .",
        "dateModified: datetime .",
        "deleted: bool .",
        "duration: int @index(int) .",
        "functions: [string] .",
        "height: int @index(int) .",
        "latitude: float @index(float) .",
        "longitude: float @index(float) .",
        "memriID: int @index(int).",
        "person_height: float @index(float) .",
        "shoulderWidth: float @index(float) .",
        "starred: bool .",
        "version: int @index(int).",
        "width: int @index(int) .",
    ];
    other_props
}

pub fn get_all_types() -> Vec<Vec<&'static str>> {
    let mut all_types = HashMap::new();
    all_types.insert(
        "address",
        vec![
            "genericType",
            "type",
            "country",
            "city",
            "street",
            "state",
            "postalCode",
            "location",
            "computeTitle",
            "version",
            "deleted",
            "starred",
            "dateCreated",
            "dateModified",
            "dateAccessed",
            "functions",
            "changelog",
            "memriID",
        ],
    );
    all_types.insert(
        "audio",
        vec![
            "name",
            "file",
            "bitrate",
            "duration",
            "genericType",
            "computeTitle",
            "includes",
            "version",
            "deleted",
            "starred",
            "dateCreated",
            "dateModified",
            "dateAccessed",
            "functions",
            "changelog",
            "memriID",
        ],
    );
    all_types.insert("AuditAccessLog", vec!["auditTarget", "dateCreated"]);
    all_types.insert(
        "company",
        vec![
            "genericType",
            "type",
            "name",
            "computeTitle",
            "version",
            "deleted",
            "starred",
            "dateCreated",
            "dateModified",
            "dateAccessed",
            "functions",
            "changelog",
            "memriID",
        ],
    );
    all_types.insert(
        "country",
        vec![
            "genericType",
            "name",
            "flag",
            "location",
            "computeTitle",
            "version",
            "deleted",
            "starred",
            "dateCreated",
            "dateModified",
            "dateAccessed",
            "functions",
            "changelog",
            "memriID",
        ],
    );
    all_types.insert(
        "dataitem",
        vec![
            "genericType",
            "computeTitle",
            "deleted",
            "starred",
            "dateCreated",
            "dateModified",
            "dateAccessed",
            "functions",
            "version",
            "changelog",
            "memriID",
        ],
    );
    all_types.insert(
        "diet",
        vec![
            "genericType",
            "type",
            "name",
            "additions",
            "version",
            "computeTitle",
            "deleted",
            "starred",
            "dateCreated",
            "dateModified",
            "dateAccessed",
            "functions",
            "changelog",
            "memriID",
        ],
    );
    all_types.insert(
        "file",
        vec![
            "uri",
            "genericType",
            "usedBy",
            "version",
            "computeTitle",
            "deleted",
            "starred",
            "dateCreated",
            "dateModified",
            "dateAccessed",
            "functions",
            "changelog",
            "memriID",
        ],
    );
    all_types.insert(
        "label",
        vec![
            "name",
            "comment",
            "color",
            "genericType",
            "computeTitle",
            "appliesTo",
            "version",
            "deleted",
            "starred",
            "dateCreated",
            "dateModified",
            "dateAccessed",
            "functions",
            "changelog",
            "memriID",
        ],
    );
    all_types.insert(
        "location",
        vec![
            "genericType",
            "latitude",
            "longitude",
            "version",
            "computeTitle",
            "deleted",
            "starred",
            "dateCreated",
            "dateModified",
            "dateAccessed",
            "functions",
            "changelog",
            "memriID",
        ],
    );
    all_types.insert(
        "logitem",
        vec![
            "date",
            "contents",
            "action",
            "genericType",
            "computeTitle",
            "appliesTo",
            "version",
            "deleted",
            "starred",
            "dateCreated",
            "dateModified",
            "dateAccessed",
            "functions",
            "changelog",
            "memriID",
        ],
    );
    all_types.insert(
        "medicalcondition",
        vec![
            "genericType",
            "type",
            "name",
            "computeTitle",
            "version",
            "deleted",
            "starred",
            "dateCreated",
            "dateModified",
            "dateAccessed",
            "functions",
            "changelog",
            "memriID",
        ],
    );
    all_types.insert(
        "note",
        vec![
            "title",
            "content",
            "genericType",
            "writtenBy",
            "sharedWith",
            "comments",
            "labels",
            "version",
            "computeTitle",
            "deleted",
            "starred",
            "dateCreated",
            "dateModified",
            "dateAccessed",
            "functions",
            "changelog",
            "memriID",
        ],
    );
    all_types.insert(
        "onlineprofile",
        vec![
            "genericType",
            "type",
            "handle",
            "computeTitle",
            "version",
            "deleted",
            "starred",
            "dateCreated",
            "dateModified",
            "dateAccessed",
            "functions",
            "changelog",
            "memriID",
        ],
    );
    all_types.insert(
        "person",
        vec![
            "firstName",
            "lastName",
            "birthDate",
            "gender",
            "sexualOrientation",
            "height",
            "shoulderWidth",
            "armLength",
            "age",
            "genericType",
            "profilePicture",
            "relations",
            "phoneNumbers",
            "websites",
            "companies",
            "addresses",
            "publicKeys",
            "onlineProfiles",
            "diets",
            "medicalConditions",
            "computeTitle",
            "version",
            "deleted",
            "starred",
            "dateCreated",
            "dateModified",
            "dateAccessed",
            "functions",
            "changelog",
            "memriID",
        ],
    );
    all_types.insert(
        "phonenumber",
        vec![
            "genericType",
            "type",
            "number",
            "computeTitle",
            "version",
            "deleted",
            "starred",
            "dateCreated",
            "dateModified",
            "dateAccessed",
            "functions",
            "changelog",
            "memriID",
        ],
    );
    all_types.insert(
        "photo",
        vec![
            "name",
            "file",
            "width",
            "height",
            "genericType",
            "computeTitle",
            "includes",
            "version",
            "deleted",
            "starred",
            "dateCreated",
            "dateModified",
            "dateAccessed",
            "functions",
            "changelog",
            "memriID",
        ],
    );
    all_types.insert(
        "publickey",
        vec![
            "genericType",
            "type",
            "name",
            "key",
            "version",
            "computeTitle",
            "deleted",
            "starred",
            "dateCreated",
            "dateModified",
            "dateAccessed",
            "functions",
            "changelog",
            "memriID",
        ],
    );
    all_types.insert(
        "video",
        vec![
            "name",
            "file",
            "width",
            "height",
            "duration",
            "genericType",
            "computeTitle",
            "includes",
            "version",
            "deleted",
            "starred",
            "dateCreated",
            "dateModified",
            "dateAccessed",
            "functions",
            "changelog",
            "memriID",
        ],
    );
    all_types.insert(
        "website",
        vec![
            "genericType",
            "type",
            "url",
            "computeTitle",
            "version",
            "deleted",
            "starred",
            "dateCreated",
            "dateModified",
            "dateAccessed",
            "functions",
            "changelog",
            "memriID",
        ],
    );
    all_types
}
