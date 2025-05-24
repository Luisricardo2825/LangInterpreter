class Teste {
  pessoa = {
    nome: "vazio",
    idade: 20,
    arrNum: [1, 2, 3],
    arrStr: [{ a: "a", b: "b" }],
    object: { a: "a", b: "b" },
    arrObj: [
      { nome: "Luis", posicao: 1 },
      { nome: "Ricardo", posicao: 2 },
    ],
  };
  static hello() {
    println("Hello, World!",len("teste"));
  }

  setNome(nome) {
    this.pessoa.nome = nome;
  }
  getNome() {
    return this.pessoa.nome;
  }

  getPessoa() {
    return this.pessoa;
  }
}

let teste = new Teste();
teste.setNome("Ricardo");

Teste.hello();
// println("Novo nome:", teste.pessoa.arrNum);

for (let element of teste.pessoa.arrObj) {
  println("Elemento:", element.nome, element.posicao);
}
