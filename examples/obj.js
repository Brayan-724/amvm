const animal = {
  raza: "Humano"
};

const persona = {
  name: "Pedro",
  ...animal
};

console.log(animal)
console.log(persona)
