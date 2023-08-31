export function shortenString(input: string): string {
	if (input.length <= 12) {
		return input
	}

	const shortened = input.substr(0, 8) + ' ... ' + input.substr(-8)
	return shortened
}
