#!/bin/sh
# Download the source documents into this directory.
cd "$(dirname "$0")" || exit 1
curl -L -o e364-laning-zierler-1954.pdf https://web.mit.edu/digitalapollo/Documents/Chapter5/laningwhirlwind.pdf
curl -L -o summer-session-1954-advanced-coding.pdf https://bitsavers.org/pdf/mit/whirlwind/summer_session_1954/Digital_Computers_Advanced_Coding_Techniques_Summer_1954.pdf
curl -L -o summary-report-37-1954q1.pdf https://bitsavers.org/pdf/mit/whirlwind/Project_Whirlwind_Summary_Report_No_37_First_Quarter_1954.pdf
curl -L -o knuth-trabb-pardo-1976-early-development.pdf http://bitsavers.org/pdf/stanford/cs_techReports/STAN-CS-76-562_EarlyDevelPgmgLang_Aug76.pdf
