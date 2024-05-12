function next(self: Iterator): number {
  return self.val += 1;
}

const iterator = { val: 0, next };

console.log("Returned: " + iterator.next(iterator) + '\n');
console.log("Returned: " + iterator.next(iterator) + '\n');
console.log("Returned: " + iterator.next(iterator) + '\n');
console.log("Returned: " + iterator.next(iterator) + '\n');
console.log("Returned: " + iterator.next(iterator) + '\n');
console.log("Returned: " + iterator.next(iterator) + '\n');
