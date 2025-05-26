class Pessoa {
  nome = null;
  idade = null;

  setNome(nome) {
    this.nome = nome;
    // this.mudouNome();
  }

  getNome() {
    return this.nome;
  }

  setIdade(idade) {
    this.idade = idade;
  }

  mudouNome() {
    println("O nome foi mudado para", this.nome);
  }
  getIdade() {
    return this.idade;
  }

  static hello() {
    println("Hello World");
  }
}

function createPessoa(nome, idade) {
  let pessoa = new Pessoa();
  pessoa.setNome(nome);
  pessoa.setIdade(idade);
  return pessoa;
}

let nomes = [
  "Ricardo",
  "João",
  "Maria",
  "José",
  "Ana",
  "Pedro",
  "Paulo",
  "Carlos",
  "Mariana",
  "Fernanda",
];
let pessoas = [];
let count = 0;
for (let nome of nomes) {
  let this_pessoa = createPessoa(nome, count + 10);

  push(pessoas, this_pessoa);
  count = count + 1;
}

pessoas[0].nome = "Ricardo Silva";
pessoas[1].setNome("João Silva");

println("Pessoas", pessoas[0].getNome());
println("this", this);
