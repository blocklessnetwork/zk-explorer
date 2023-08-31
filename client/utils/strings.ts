export function shortenString(input: string, len = 8): string {
	if (input.length <= len * 2) {
		return input
	}

	const shortened = input.substr(0, len) + ' ...' + input.substr(-len)
	return shortened
}

export function detectUUIDs(input: string): boolean {
	const uuidPattern = /[0-9a-fA-F]{8}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{12}/g

	return uuidPattern.test(input)
}
