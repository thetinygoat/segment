package text

import (
	"strings"
	"unicode"
)

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

// NumFilter filters out numbers
func NumFilter(input string) string {
	var b strings.Builder

	for _, c := range input {
		if unicode.IsDigit(c) {
			continue
		}
		b.WriteRune(c)
	}
	return b.String()
}
