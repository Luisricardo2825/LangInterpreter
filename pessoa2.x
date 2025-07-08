function ola(nome) {
    Io.println("Olá", nome);
}

export default class Pessoa {
    nome = "";
    idade = 0;
    constructor(self, nome, idade) {
        ola(nome);
        self.nome = nome;
        self.idade = idade;
    }

    toString(self) {
        return "Nome: " + self.nome + " Idade: " + self.idade;
    }
}
