let msg = "Esse é o nome externo";

function getName() {
    return msg;
}
class Teste {
    msg = "Esse é o nome interno";

    getName(msg) {
        return msg;
    }
}

let obj = new Teste();
Io.println(obj.getName("teste"), getName());
