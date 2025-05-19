let num = toNumber(input("Digite um número: "))

println(typeof(num))

if (num==1) {
    println("é igual a",num)
}else if(num==2){
    println("é igual a ",num)
}else if(num==3){
    println("é igual a ",num)
}else{
    println("Não encontrado: ",num)
}

let obj = {a:num, b:num*2, c:num*3}

println("Objeto: ",obj)