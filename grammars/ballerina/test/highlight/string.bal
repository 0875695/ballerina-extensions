function test() {
    var a = ["ab", "bc"];
//            ^ string
//                  ^ string
    var b = ["ab", 1, "bc"];
//            ^ string
//                 ^ number
//                     ^ string
    var c = "ab" + "cd";
//           ^ string
//               ^ operator
//                  ^ string

    var d = `SELECT "COUNTRY#R" FROM country`;
//           ^ string

    var e = `SELECT * FROM ${tableName} WHERE id = 1`;
//           ^ string
//                          ^ punctuation.special
//                           ^ variable
//                                    ^ punctuation.special
}

