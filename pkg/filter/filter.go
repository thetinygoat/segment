package filter

func Punctuation(tokens []string) []string {
	var filtered []string

	for _, token := range tokens {
		switch token {
		case ".", "?", "!", ",", ":", ";", "-", "[", "]", "{", "}", "(", ")", "\"", "'", "—":
			continue
		default:
			filtered = append(filtered, token)
		}
	}

	return filtered
}
