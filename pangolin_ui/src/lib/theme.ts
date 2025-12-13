export const lightTheme = {
  primary: '#1976d2',
  secondary: '#dc004e',
  background: '#ffffff',
  surface: '#f5f5f5',
  error: '#b00020',
  onPrimary: '#ffffff',
  onSecondary: '#ffffff',
  onBackground: '#000000',
  onSurface: '#000000',
  onError: '#ffffff',
};

export const darkTheme = {
  primary: '#90caf9',
  secondary: '#f48fb1',
  background: '#121212',
  surface: '#1e1e1e',
  error: '#cf6679',
  onPrimary: '#000000',
  onSecondary: '#000000',
  onBackground: '#ffffff',
  onSurface: '#ffffff',
  onError: '#000000',
};

export type Theme = typeof lightTheme;

export const applyTheme = (theme: Theme) => {
  if (typeof document === 'undefined') return;
  const root = document.documentElement;
  Object.entries(theme).forEach(([key, value]) => {
    // Convert camelCase to kebab-case for CSS variables
    const cssVar_name = `--md-sys-color-${key.replace(/([a-z0-9]|(?=[A-Z]))([A-Z])/g, '$1-$2').toLowerCase()}`;
    root.style.setProperty(cssVar_name, value);
  });
};
