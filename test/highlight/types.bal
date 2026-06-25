type A int|boolean;

type R record {
    @annot
//  ^ attribute
    A a;
//  ^ type
    float b;
//  ^ type.builtin
};

function tmp (int a, boolean b) returns int {
//             ^ type.builtin
//                    ^ type.builtin
//                                       ^ type.builtin
    int c = a + 1;
//   ^ type.builtin
    boolean b = false;
//   ^ type.builtin
    A x = 1;
//  ^ type
    int&byte y = 5;
//  ^ type.builtin
//      ^ type.builtin
    return c;
}

@openapi:ServiceInfo {
// <- attribute
// ^ module
//        ^ attribute
    embed: true
}
class Dummy {
    @annot
//  ^ attribute
    int val = 1;

    @annot
//  ^ attribute
    function method() {}
}


