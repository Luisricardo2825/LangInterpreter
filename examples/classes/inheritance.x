class Pessoa {
    nome;
    idade;
    constructor() {
        this.nome = "João";
        this.idade = 20;
    }
    getNome() {
        return this.nome;
    }
    getIdade() {
        return this.idade;
    }
}
class Funcionario extends Pessoa {
    salario;
    sup;
    constructor() {
        this.sup = new super();
        this.salario = 1000;
    }

    getSalario() {
        return this.salario;
    }
}

let pessoa = new Pessoa();
let funcionario = new Funcionario();

function checkType(value) {
    if (value instanceof Pessoa) {
        Io.println("É uma pessoa");
    } else {
        Io.println("Não é uma pessoa");
    }
    if (value instanceof Funcionario) {
        Io.println("É um funcionário, nome: ", value.sup.getNome(), " salario: ", value.getSalario());
    } else {
        Io.println("Não é um funcionário");
    }
}

checkType(funcionario);
Io.println("-------------------");
checkType(pessoa);
