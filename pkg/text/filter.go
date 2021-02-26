package text

import "strings"

// FilterPuncChar filters punctuations
func FilterPuncChar(input string) string {
	var b strings.Builder
	for _, c := range input {
		switch c {
		case '.', '?', '!':
			continue
		case ',', ';', ':':
			continue
		case '-', '—':
			continue
		case '(', ')', '[', ']', '{', '}':
			continue
		case '\'', '"', '*':
			continue
		}
		b.WriteRune(c)
	}

	return b.String()
}
