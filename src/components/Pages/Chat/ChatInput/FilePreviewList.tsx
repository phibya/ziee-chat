import { Divider } from 'antd'
import { FileCard } from '../../../common/FileCard'
import type { File as FileType } from '../../../../types'
import type { FileUploadProgress } from '../../../../types/client/file'

interface FilePreviewListProps {
  files: Map<string, FileType>
  newFiles: Map<string, FileType>
  uploadingFiles: Map<string, FileUploadProgress>
  onRemoveFile: (fileId: string) => void
}

export const FilePreviewList = ({
  files,
  newFiles,
  uploadingFiles,
  onRemoveFile,
}: FilePreviewListProps) => {
  const hasFiles =
    files.size > 0 || newFiles.size > 0 || uploadingFiles.size > 0

  if (!hasFiles) return null

  return (
    <>
      <Divider style={{ margin: 0 }} />
      <div style={{ padding: '8px' }}>
        <div className="flex gap-2 w-full overflow-x-auto">
          {Array.from(files.values()).map(file => (
            <div key={file.id} className={'flex-1 min-w-20 max-w-24'}>
              <FileCard
                file={file}
                canDelete={false}
                canRemove={true}
                onRemove={onRemoveFile}
              />
            </div>
          ))}
          {Array.from(newFiles.values()).map(file => (
            <div key={file.id} className={'flex-1 min-w-20 max-w-24'}>
              <FileCard file={file} canDelete={true} onDelete={onRemoveFile} />
            </div>
          ))}
          {Array.from(uploadingFiles.values()).map(uploadingFile => (
            <div key={uploadingFile.id} className={'flex-1 min-w-20 max-w-24'}>
              <FileCard
                uploadingFile={{
                  id: uploadingFile.id,
                  filename: uploadingFile.filename,
                  progress: uploadingFile.progress || 0,
                  status: 'uploading',
                  size: uploadingFile.size,
                }}
                onRemove={onRemoveFile}
              />
            </div>
          ))}
        </div>
      </div>
    </>
  )
}
