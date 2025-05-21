class Teste {
  nome = "vazio";

  setNome(nome) {
    this.nome = nome;
  }

  getNome() {
    return this.nome;
  }
}

let teste = new Teste();
teste.setNome("Ricardo");

println(teste.getNome());
