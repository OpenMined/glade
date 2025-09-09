# glade

## Data Preprocessing
1. Requires `uv` to be installed.
https://github.com/astral-sh/uv

```bash
# On macOS and Linux.
curl -LsSf https://astral.sh/uv/install.sh | sh
```

2. Go to notebooks and run:  
```bash
cd notebooks
./jupyter.sh
```

3. Run the Jupyter notebooks in sequence:  
    - `notebooks/1_download.ipynb`
    - `notebooks/2_check_positions.ipynb`
    - `notebooks/3_clinvar_filter.ipynb`

4. Output is in:  
```bash
./notebooks/results
```