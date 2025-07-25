import { create } from "zustand";
import { Repository } from "../../types/api/repository";

interface RepositoryDrawerState {
  open: boolean;
  loading: boolean;
  editingRepository: Repository | null;
}

export const useRepositoryDrawerStore = create<RepositoryDrawerState>(() => ({
  open: false,
  loading: false,
  editingRepository: null,
}));

// Modal actions
export const openRepositoryDrawer = (repository?: Repository) => {
  useRepositoryDrawerStore.setState({
    open: true,
    editingRepository: repository || null,
  });
};

export const closeRepositoryDrawer = () => {
  useRepositoryDrawerStore.setState({
    open: false,
    loading: false,
    editingRepository: null,
  });
};

export const setRepositoryDrawerLoading = (loading: boolean) => {
  useRepositoryDrawerStore.setState({
    loading,
  });
};
