export async function importAsDefault<T>(
  i: Promise<{ [key: string]: T }>,
  name: string
): Promise<{ default: T }> {
  return { default: (await i)[name] };
}
