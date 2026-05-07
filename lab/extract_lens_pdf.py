import sys
try:
    import pypdf
    reader = pypdf.PdfReader('docs/Constellation_Lens_Concept_Paper_Eisa.pdf')
    for i, page in enumerate(reader.pages):
        print(f'--- Page {i+1} ---')
        print(page.extract_text())
        print()
except ImportError:
    try:
        from pypdf2 import PdfReader
    except ImportError:
        try:
            from PyPDF2 import PdfReader
            reader = PdfReader('docs/Constellation_Lens_Concept_Paper_Eisa.pdf')
            for i, page in enumerate(reader.pages):
                print(f'--- Page {i+1} ---')
                print(page.extract_text())
                print()
        except ImportError:
            print('No PDF library available')
            sys.exit(1)
