import { Set } from "./examples/Native/Array.x";

let listaUnica = new Set(3, 2, 4, 1);
let num = new Number(10);

listaUnica.push(11);
listaUnica.push(11);
listaUnica.push(12);
listaUnica.push("Teste");
listaUnica.push("-1");
listaUnica.push(11);
listaUnica.push(11);
// lista.push(13);
listaUnica.sort();

let object = { b: 10, c: "Teste", a: "Valor de A", $: 15, "[]": "Brackets([])" };
object.a = 0;
object.c = 20;
object["%"] = 10;

Io.println("Lista:", listaUnica, object, object["[]"], listaUnica.push, Array.push, num);
