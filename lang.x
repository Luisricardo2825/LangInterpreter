function getNum() {
  return 19;
}

function getAge() {
  return getNum();
}
let a = {
  name: "John",
  age: 30,
  city: "New York",
  getAge: getAge,
};

print(a.getAge() + 1 < 10);
