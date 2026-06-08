type DiiaClientEntity record {
    @sql:Column {name: "CLIENTID"}
    decimal clientId;
    @sql:Column {name: "COUNTRY"}
    json country;
    @sql:Column {name: "LASTNAMEUA"}
    string? lastNameUA;
};

configurable string USER = ?;
configurable string PASSWORD = ?;
configurable string HOST = ?;
configurable int PORT = ?;
configurable string DATABASE = ?;

final postgresql:Client dbClient = check new (
    host = HOST,
    port = PORT
);

listener http:Listener diiaService = new (9090);

public function testDoFail() returns error? {
    do {
        int age = -5;
        if age < 0 {
            fail error("Age cannot be negative", age = age);
        }
    } on fail var e {
        if e is sql:NoRowsError {
            return;
        }
        return e;
    }
}
