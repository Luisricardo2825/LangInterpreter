import { Date } from "./examples/classes/Date.x";
class Pessoa {
    nome;
    idade;
    dataNascimento;
    constructor(nome, idade, dataNascimento) {
        this.nome = nome;
        this.idade = idade;
        this.dataNascimento = dataNascimento;
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
    setIdade(idade) {
        this.idade = idade;
    }
    getDataNascimento() {
        return this.dataNascimento;
    }
    setDataNascimento(dataNascimento) {
        this.dataNascimento = dataNascimento;
    }
    static ola() {
        Io.println("Olá");
    }
}

class Funcionario extends Pessoa {
    salario = 0;
    constructor(nome, idade, dataNascimento, salario) {
        super(nome, idade, dataNascimento);
        this.salario = salario;
    }
    static fromPessoa(pessoa, salario) {
        return new Funcionario(
            pessoa.getNome(),
            pessoa.getIdade(),
            pessoa.getDataNascimento(),
            salario
        );
    }
}

let pessoas = [
    new Pessoa("João", 20, new Date(2000, 1, 1)),
    new Pessoa("Maria", 30, new Date(1990, 1, 1)),
    new Pessoa("José", 40, new Date(1980, 1, 1)),
    new Pessoa("Ana", 50, new Date(1970, 1, 1)),
    new Pessoa("Pedro", 60, new Date(1960, 1, 1)),
    new Pessoa("Paulo", 70, new Date(1950, 1, 1)),
    new Pessoa("Carlos", 80, new Date(1940, 1, 1)),
    new Pessoa("Mariana", 90, new Date(1930, 1, 1)),
    new Pessoa("Marta", 100, new Date(1920, 1, 1)),
    new Pessoa("Mariana", 110, new Date(1910, 1, 1)),
];

let funcionarios = [];
let count = 1;
for (let pessoa of pessoas) {
    let funcionario = Funcionario.fromPessoa(pessoa, 2000 * count);
    funcionarios.push(funcionario);
    count++;
}

Io.println(funcionarios);
