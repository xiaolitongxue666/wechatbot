import type { Ref } from "vue";

export function assignIfChanged<T>(target: Ref<T>, next: T, equals: (left: T, right: T) => boolean = Object.is): boolean {
  if (equals(target.value, next)) {
    return false;
  }
  target.value = next;
  return true;
}

export function jsonEquals<T>(left: T, right: T): boolean {
  return JSON.stringify(left) === JSON.stringify(right);
}

export function linesEquals(left: string[] | undefined, right: string[]): boolean {
  const previous = left ?? [];
  if (previous.length !== right.length) {
    return false;
  }
  return previous.every((line, index) => line === right[index]);
}
