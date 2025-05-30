class Pessoa {
  nome;
  idade;

  constructor(nome) {
    this.nome = nome;
    this.idade = 20;
  }
  constructor(nome, idade) {
    this.nome = nome;
    this.idade = idade;
  }

  getNome() {
    return this.nome;
  }
  getIdade() {
    return this.idade;
  }
  setNome(nome) {
    this.nome = nome;
  }
}