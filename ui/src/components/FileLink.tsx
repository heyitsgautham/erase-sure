interface FileLinkProps {
  path: string;
  label?: string;
  type?: 'json' | 'pdf' | 'folder';
}

function FileLink({ path, label, type = 'folder' }: FileLinkProps) {
  const openFile = async () => {
    try {
      // In production, use Tauri's shell API to open files/folders
      // await open(path);
      console.log('Opening:', path);
    } catch (error) {
      console.error('Failed to open file:', error);
    }
  };

  const getIcon = () => {
    switch (type) {
      case 'json':
        return '📄';
      case 'pdf':
        return '📋';
      case 'folder':
      default:
        return '📁';
    }
  };

  return (
    <button 
      className="btn btn-secondary" 
      onClick={openFile}
      style={{
        display: 'flex',
        alignItems: 'center',
        gap: '0.5rem',
        textAlign: 'left',
        minWidth: '200px'
      }}
    >
      <span>{getIcon()}</span>
      <div style={{ flex: 1, minWidth: 0 }}>
        <div className="font-medium">{label || path}</div>
        {label && (
          <div className="text-xs" style={{ opacity: 0.7, overflow: 'hidden', textOverflow: 'ellipsis', whiteSpace: 'nowrap' }}>
            {path}
          </div>
        )}
      </div>
    </button>
  );
}

export default FileLink;
