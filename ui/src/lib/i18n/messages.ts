export type MessagePartType = {
  text: string;
  bold?: boolean;
  code?: boolean;
};

export type MessageType = string | MessagePartType[];

export const messageParts = (content: MessageType): MessagePartType[] =>
  typeof content === "string" ? [{ text: content }] : content;

export const containsAccountName = (content: MessageType, name: string): boolean => {
  return Array.isArray(content) && content.some((part) => part.bold && part.text === name);
};
