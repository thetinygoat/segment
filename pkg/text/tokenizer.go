package text

import (
	"strings"
)

// WhitespaceTokenizer tokenizes the input string on whitespace
func WhitespaceTokenizer(input string) []string {
	// return strings.Split(input, " ")
	var b strings.Builder
	var res []string
	for _, c := range input {
		switch c {
		case ' ':

			if len(b.String()) > 0 {
				res = append(res, b.String())
			}
			b.Reset()
		default:
			b.WriteRune(c)
		}
	}
	if len(b.String()) > 0 {
		res = append(res, b.String())
	}
	return res
}
