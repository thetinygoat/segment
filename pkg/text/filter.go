package text

import "strings"

// PuncFilter filters punctuations
func PuncFilter(input string) string {
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
