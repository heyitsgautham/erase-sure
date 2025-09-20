import { useState, useCallback } from 'react';
import { invoke } from '@tauri-apps/api/tauri';

export interface FileSystemEntry {
  name: string;
  path: string;
  is_dir: boolean;
  size?: number;
  modified?: string;
}

export interface DirectoryListing {
  entries: FileSystemEntry[];
  total_size: number;
  total_items: number;
}

export interface UseFileBrowserReturn {
  currentPath: string;
  listing: DirectoryListing | null;
  selectedPaths: string[];
  isLoading: boolean;
  error: string | null;
  
  // Navigation
  navigateTo: (path: string) => Promise<void>;
  navigateUp: () => Promise<void>;
  goHome: () => Promise<void>;
  
  // Selection
  toggleSelection: (path: string) => void;
  selectAll: () => void;
  clearSelection: () => void;
  
  // Size calculation
  calculateSelectionSize: () => Promise<number>;
  
  // Utilities
  formatFileSize: (bytes: number) => string;
  isSelected: (path: string) => boolean;
}

export const useFileBrowser = (): UseFileBrowserReturn => {
  const [currentPath, setCurrentPath] = useState<string>('');
  const [listing, setListing] = useState<DirectoryListing | null>(null);
  const [selectedPaths, setSelectedPaths] = useState<string[]>([]);
  const [isLoading, setIsLoading] = useState<boolean>(false);
  const [error, setError] = useState<string | null>(null);

  const navigateTo = useCallback(async (path: string) => {
    setIsLoading(true);
    setError(null);
    
    try {
      const result = await invoke<DirectoryListing>('browse_folders', { path });
      setListing(result);
      setCurrentPath(path);
      setSelectedPaths([]); // Clear selection when navigating
    } catch (err) {
      setError(err as string);
      console.error('Failed to browse folder:', err);
    } finally {
      setIsLoading(false);
    }
  }, []);

  const navigateUp = useCallback(async () => {
    if (!currentPath) return;
    
    const parentPath = currentPath.split('/').slice(0, -1).join('/') || '/';
    await navigateTo(parentPath);
  }, [currentPath, navigateTo]);

  const goHome = useCallback(async () => {
    setIsLoading(true);
    setError(null);
    
    try {
      console.log('Calling browse_folders with path: null (should get home directory)');
      const result = await invoke<DirectoryListing>('browse_folders', { path: null });
      console.log('browse_folders result:', result);
      setListing(result);
      
      // Set current path - our Rust function should return home directory when path is null
      // We'll try to determine the actual path from the environment or use a fallback
      const homePath = '/home/kinux'; // Using the actual user path for now
      setCurrentPath(homePath);
      setSelectedPaths([]);
    } catch (err) {
      const errorMessage = typeof err === 'string' ? err : String(err);
      setError(`Failed to load home directory: ${errorMessage}`);
      console.error('Failed to navigate to home:', err);
      console.error('Error details:', err);
    } finally {
      setIsLoading(false);
    }
  }, []);

  const toggleSelection = useCallback((path: string) => {
    setSelectedPaths(prev => {
      if (prev.includes(path)) {
        return prev.filter(p => p !== path);
      } else {
        return [...prev, path];
      }
    });
  }, []);

  const selectAll = useCallback(() => {
    if (!listing) return;
    
    const allPaths = listing.entries.map(entry => entry.path);
    setSelectedPaths(allPaths);
  }, [listing]);

  const clearSelection = useCallback(() => {
    setSelectedPaths([]);
  }, []);

  const calculateSelectionSize = useCallback(async (): Promise<number> => {
    if (selectedPaths.length === 0) return 0;
    
    try {
      const size = await invoke<number>('calculate_selection_size', { paths: selectedPaths });
      return size;
    } catch (err) {
      console.error('Failed to calculate selection size:', err);
      return 0;
    }
  }, [selectedPaths]);

  const formatFileSize = useCallback((bytes: number): string => {
    if (bytes === 0) return '0 B';
    
    const k = 1024;
    const sizes = ['B', 'KB', 'MB', 'GB', 'TB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    
    return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
  }, []);

  const isSelected = useCallback((path: string): boolean => {
    return selectedPaths.includes(path);
  }, [selectedPaths]);

  return {
    currentPath,
    listing,
    selectedPaths,
    isLoading,
    error,
    
    navigateTo,
    navigateUp,
    goHome,
    
    toggleSelection,
    selectAll,
    clearSelection,
    
    calculateSelectionSize,
    
    formatFileSize,
    isSelected,
  };
};
