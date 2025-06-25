# LangInterpreter

[![Build](https://img.shields.io/badge/build-passing-brightgreen)](https://github.com/Luisricardo2825/LangInterpreter)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](./LICENSE)
[![Rust](https://img.shields.io/badge/rust-stable-orange)](https://www.rust-lang.org)

**LangInterpreter** é um interpretador de linguagem esotérica, criado com o propósito de estudo e aprendizado da construção de linguagens de programação e interpretadores em Rust.

## 🎯 Objetivo

Este projeto tem como objetivo:

-   Desenvolver um interpretador completo em Rust
-   Aprender sobre léxicos (lexers), sintaxe (parsers), ASTs e ambientes de execução
-   Explorar conceitos como escopo, closures, chamadas de função, e estrutura de objetos

## 🧠 Funcionalidades

-   Interpretador com suporte a:
    -   Variáveis e escopos aninhados
    -   Blocos de código
    -   Funções com argumentos e retorno
    -   Estruturas de controle (`if`, `for`, `while`)
    -   Suporte a objetos e arrays
    -   Destructuring
    -   Módulos e sistema de import/export
-   Parser recursivo descendente
-   Ambiente com escopos usando `Rc<RefCell<Environment>>`
-   Mensagens de erro detalhadas com `ariadne`

## 🛠 Tecnologias

-   **Rust** — linguagem de implementação
-   [`logos`](https://docs.rs/logos) — lexer
-   [`ariadne`](https://docs.rs/ariadne) — erros com destaque
-   [`regex`](https://docs.rs/regex) — suporte a padrões
-   [`serde`](https://docs.rs/serde), [`serde_json`](https://docs.rs/serde_json) — suporte a serialização
-   [`anyhow`](https://docs.rs/anyhow) — tratamento de erros

## 🚧 Instalação

A instalação ainda não está disponível de forma automatizada. O projeto está em fase inicial e voltado para experimentações locais.

## ▶️ Como Usar

Você pode ver exemplos de uso na pasta [`./examples`](./examples) na raiz do projeto. Os arquivos contêm scripts que podem ser usados para testes e exploração da linguagem.

## 🤝 Contribuindo

Atualmente, o projeto é pequeno e sem foco em uso prático. Contribuições não estão sendo ativamente solicitadas, mas sinta-se livre para abrir issues ou forks se desejar explorar ou discutir funcionalidades.

## 📄 Licença

Este projeto é de uso **livre**.

---

> Criado com fins educacionais por [Luis Ricardo](https://github.com/Luisricardo2825)
