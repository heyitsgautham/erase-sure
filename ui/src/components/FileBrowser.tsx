import React, { useEffect, useState } from 'react';
import { invoke } from '@tauri-apps/api/tauri';
import { useFileBrowser, FileSystemEntry } from '../hooks/useFileBrowser';

interface FileBrowserProps {
  onSelectionChange: (selectedPaths: string[]) => void;
  multiSelect?: boolean;
  allowFiles?: boolean;
  allowFolders?: boolean;
  maxSelectionSize?: number; // in bytes
  title?: string;
}

const FileBrowser: React.FC<FileBrowserProps> = ({
  onSelectionChange,
  multiSelect = true,
  allowFiles = true,
  allowFolders = true,
  maxSelectionSize,
  title = "Select Files and Folders"
}) => {
  const {
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
  } = useFileBrowser();

  const [selectionSize, setSelectionSize] = useState<number>(0);
  const [showSizeWarning, setShowSizeWarning] = useState<boolean>(false);
  const [folderSizes, setFolderSizes] = useState<Map<string, number>>(new Map());

  // Initialize by navigating to home
  useEffect(() => {
    console.log('FileBrowser mounting, calling goHome()');
    goHome();
  }, [goHome]);

  // Update parent component when selection changes
  useEffect(() => {
    onSelectionChange(selectedPaths);
  }, [selectedPaths, onSelectionChange]);

  // Calculate selection size when paths change
  useEffect(() => {
    if (selectedPaths.length > 0) {
      calculateSelectionSize().then(size => {
        setSelectionSize(size);
        setShowSizeWarning(maxSelectionSize ? size > maxSelectionSize : false);
      });
    } else {
      setSelectionSize(0);
      setShowSizeWarning(false);
    }
  }, [selectedPaths, calculateSelectionSize, maxSelectionSize, folderSizes]);

  const handleEntryClick = (entry: FileSystemEntry) => {
    if (entry.is_dir) {
      navigateTo(entry.path);
    }
  };

  const handleSelectionToggle = async (entry: FileSystemEntry, event: React.MouseEvent) => {
    event.stopPropagation();
    
    // Check if this type of entry is allowed for selection
    if ((entry.is_dir && !allowFolders) || (!entry.is_dir && !allowFiles)) {
      return;
    }

    // If it's a folder and we don't have its size yet, calculate it
    if (entry.is_dir && !folderSizes.has(entry.path)) {
      setFolderSizes(prev => new Map(prev).set(entry.path, -1)); // -1 = calculating
      try {
        const actualSize = await invoke<number>('calculate_selection_size', { paths: [entry.path] });
        setFolderSizes(prev => new Map(prev).set(entry.path, actualSize));
      } catch (error) {
        console.error('Failed to calculate folder size:', error);
        setFolderSizes(prev => new Map(prev).set(entry.path, 0));
      }
    }

    if (!multiSelect) {
      // Single select mode - clear others first
      if (!isSelected(entry.path)) {
        onSelectionChange([entry.path]);
      } else {
        onSelectionChange([]);
      }
      return;
    }

    toggleSelection(entry.path);
  };

  const getPathParts = () => {
    if (!currentPath) return [];
    return currentPath.split('/').filter(part => part !== '');
  };

  const navigateToPathPart = (index: number) => {
    const parts = getPathParts();
    const newPath = '/' + parts.slice(0, index + 1).join('/');
    navigateTo(newPath);
  };

  const renderBreadcrumbs = () => {
    const parts = getPathParts();
    
    return (
      <div className="flex items-center space-x-2 mb-4 text-sm">
        <button
          onClick={goHome}
          className="px-2 py-1 bg-blue-100 hover:bg-blue-200 rounded text-blue-700"
        >
          🏠 Home
        </button>
        
        {parts.map((part, index) => (
          <span key={index}>
            <span className="text-gray-400">/</span>
            <button
              onClick={() => navigateToPathPart(index)}
              className="px-2 py-1 hover:bg-gray-100 rounded text-gray-700"
            >
              {part}
            </button>
          </span>
        ))}
      </div>
    );
  };

  const renderEntry = (entry: FileSystemEntry) => {
    const selected = isSelected(entry.path);
    const canSelect = (entry.is_dir && allowFolders) || (!entry.is_dir && allowFiles);
    
    return (
      <div
        key={entry.path}
        className={`flex items-center p-2 rounded cursor-pointer hover:bg-gray-50 ${
          selected ? 'bg-blue-100 border border-blue-300' : ''
        }`}
        onClick={() => handleEntryClick(entry)}
      >
        {canSelect && (
          <input
            type="checkbox"
            checked={selected}
            onChange={(e) => handleSelectionToggle(entry, e as any)}
            className="mr-3"
            onClick={(e) => e.stopPropagation()}
          />
        )}
        
        <div className="flex items-center flex-1">
          <span className="mr-2 text-lg">
            {entry.is_dir ? '📁' : '📄'}
          </span>
          
          <div className="flex-1">
            <div className="font-medium text-gray-900">{entry.name}</div>
            <div className="text-sm text-gray-500">
              {!entry.is_dir && entry.size && formatFileSize(entry.size)}
              {entry.is_dir && folderSizes.has(entry.path) && (
                <span>
                  {folderSizes.get(entry.path) === -1 
                    ? 'Calculating size...' 
                    : formatFileSize(folderSizes.get(entry.path) || 0)
                  }
                </span>
              )}
              {entry.modified && (
                <span className="ml-2">Modified: {entry.modified}</span>
              )}
            </div>
          </div>
        </div>
      </div>
    );
  };

  if (isLoading && !listing) {
    return (
      <div className="flex items-center justify-center h-64">
        <div className="text-gray-500">Loading...</div>
      </div>
    );
  }

  return (
    <div className="bg-white border rounded-lg shadow-sm">
      {/* Header */}
      <div className="p-4 border-b">
        <div className="flex items-center justify-between mb-2">
          <h3 className="text-lg font-semibold">{title}</h3>
          <div className="flex space-x-2">
            {selectedPaths.length > 0 && (
              <button
                onClick={clearSelection}
                className="px-3 py-1 text-sm bg-gray-100 hover:bg-gray-200 rounded"
              >
                Clear Selection
              </button>
            )}
            {multiSelect && listing && listing.entries.length > 0 && (
              <button
                onClick={selectAll}
                className="px-3 py-1 text-sm bg-blue-100 hover:bg-blue-200 rounded text-blue-700"
              >
                Select All
              </button>
            )}
          </div>
        </div>
        
        {renderBreadcrumbs()}
        
        {/* Navigation */}
        <div className="flex items-center space-x-2">
          <button
            onClick={navigateUp}
            disabled={!currentPath || currentPath === '/'}
            className="px-3 py-1 bg-gray-100 hover:bg-gray-200 disabled:opacity-50 disabled:cursor-not-allowed rounded"
          >
            ⬆️ Up
          </button>
          
          <div className="text-sm text-gray-600">
            {listing && `${listing.total_items} items`}
            {listing && listing.total_size > 0 && ` • ${formatFileSize(listing.total_size)}`}
          </div>
        </div>
      </div>

      {/* Selection Info */}
      {selectedPaths.length > 0 && (
        <div className="p-3 bg-blue-50 border-b">
          <div className="flex items-center justify-between">
            <div className="text-sm">
              <span className="font-medium">{selectedPaths.length}</span> item(s) selected
              {selectionSize > 0 && (
                <span className="ml-2 text-gray-600">
                  • {formatFileSize(selectionSize)}
                </span>
              )}
            </div>
            
            {showSizeWarning && maxSelectionSize && (
              <div className="text-sm text-amber-600">
                ⚠️ Selection exceeds {formatFileSize(maxSelectionSize)}
              </div>
            )}
          </div>
        </div>
      )}

      {/* Error Display */}
      {error && (
        <div className="p-3 bg-red-50 border-b text-red-700 text-sm">
          Error: {error}
        </div>
      )}

      {/* File Listing */}
      <div className="max-h-96 overflow-y-auto">
        {listing && listing.entries.length > 0 ? (
          <div className="p-2 space-y-1">
            {listing.entries.map(renderEntry)}
          </div>
        ) : (
          <div className="p-8 text-center text-gray-500">
            {isLoading ? 'Loading...' : 'No items found'}
          </div>
        )}
      </div>
    </div>
  );
};

export default FileBrowser;
