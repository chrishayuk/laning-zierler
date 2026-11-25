#!/bin/sh
# Download the source documents into this directory. A file already here is
# left alone, and a download that fails (bitsavers.org refuses some clients)
# is tried again from a mirror, never saved as an error page.
cd "$(dirname "$0")" || exit 1

get() {
    file=$1
    shift
    if [ -s "$file" ]; then
        echo "have $file"
        return
    fi
    for url in "$@"; do
        if curl -fsL -o "$file.part" "$url" && head -c 5 "$file.part" | grep -q '%PDF'; then
            mv "$file.part" "$file"
            echo "got $file"
            return
        fi
    done
    rm -f "$file.part"
    echo "could not get $file" >&2
}

B=https://bitsavers.org/pdf
M=http://bitsavers.trailing-edge.com/pdf
K=https://archive.computerhistory.org/resources/text/Knuth_Don_X4100/PDF_index/k-3-pdf

get e364-laning-zierler-1954.pdf https://web.mit.edu/digitalapollo/Documents/Chapter5/laningwhirlwind.pdf
get summer-session-1954-advanced-coding.pdf \
    $B/mit/whirlwind/summer_session_1954/Digital_Computers_Advanced_Coding_Techniques_Summer_1954.pdf \
    $M/mit/whirlwind/summer_session_1954/Digital_Computers_Advanced_Coding_Techniques_Summer_1954.pdf
get summary-report-37-1954q1.pdf \
    $B/mit/whirlwind/Project_Whirlwind_Summary_Report_No_37_First_Quarter_1954.pdf \
    $M/mit/whirlwind/Project_Whirlwind_Summary_Report_No_37_First_Quarter_1954.pdf
get knuth-trabb-pardo-1976-early-development.pdf \
    $B/stanford/cs_techReports/STAN-CS-76-562_EarlyDevelPgmgLang_Aug76.pdf \
    $M/stanford/cs_techReports/STAN-CS-76-562_EarlyDevelPgmgLang_Aug76.pdf
get laning-to-knuth-1976-07-02.pdf https://cs.stanford.edu/~knuth/from-Laning-76-07-02.pdf
get knuth-to-laning-1976-08-16.pdf https://cs.stanford.edu/~knuth/to-Laning-76-08-16.pdf
get laning-1953-listing.pdf $K/k-3-u2774-Whirlwind-program.pdf
get knuth-1976-notes-on-1953-listing.pdf $K/k-3-u2774-Whirlwind-notes.pdf
