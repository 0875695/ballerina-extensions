@openapi:ServiceInfo {
    embed: true,
    title: "Сервіси ГСЦ для ДІЇ",
    version: "0.1.0"
}
isolated service http:Service / on diiaService {
    @openapi:ResourceInfo {summary: "Отримання посвідчення"}
    isolated resource function get diia/'client/[int inn] () returns error? {
    }
}
